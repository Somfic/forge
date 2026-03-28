use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::OnceLock;
use std::time::Instant;

use tokio::sync::Mutex;

static SESSIONS: OnceLock<Mutex<HashMap<String, HlsSession>>> = OnceLock::new();

pub struct HlsSession {
    pub dir: PathBuf,
    pub child: Option<tokio::process::Child>,
    pub last_access: Instant,
}

fn sessions() -> &'static Mutex<HashMap<String, HlsSession>> {
    SESSIONS.get_or_init(|| Mutex::new(HashMap::new()))
}

/// Generate a random 16-char hex session ID.
fn new_session_id() -> String {
    use std::time::SystemTime;
    let t = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    // Mix time with a random-ish value from stack address
    let stack_val = &t as *const _ as usize;
    format!("{:08x}{:08x}", t as u32, stack_val as u32)
}

/// Start an HLS remux session. Returns (session_id, playlist_path).
/// Spawns ffmpeg to write HLS segments to a temp directory.
/// If `start_time` > 0, ffmpeg seeks to that position before encoding.
pub async fn start_session(
    storage: &forge::Storage,
    input_path: &std::path::Path,
    audio_index: usize,
    start_time: f64,
) -> forge::Result<(String, String)> {
    let session_id = new_session_id();
    let dir = storage.join(format!("hls/{session_id}"));
    tokio::fs::create_dir_all(&dir).await?;

    let playlist_path = dir.join("playlist.m3u8");
    let segment_pattern = dir.join("seg%05d.ts");

    let mut pre_args: Vec<String> = Vec::new();
    if start_time > 0.0 {
        pre_args.extend_from_slice(&[
            "-ss".into(),
            format!("{start_time:.3}"),
            "-copyts".into(),
        ]);
    }

    let child = tokio::process::Command::new("ffmpeg")
        .args(&pre_args)
        .args([
            "-i",
            input_path.to_str().unwrap_or(""),
            "-map",
            "0:v:0",
            "-map",
            &format!("0:a:{audio_index}"),
            "-c:v",
            "copy",
            "-c:a",
            "aac",
            "-b:a",
            "192k",
            "-ac",
            "2",
            "-af",
            "aresample=async=1:first_pts=0",
            "-f",
            "hls",
            "-hls_time",
            "4",
            "-hls_list_size",
            "0",
            "-hls_flags",
            "append_list",
            "-hls_segment_filename",
            segment_pattern.to_str().unwrap_or(""),
            "-hls_playlist_type",
            "event",
        ])
        .arg(playlist_path.to_str().unwrap_or(""))
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
        .map_err(|e| forge::Error::Generic(format!("Failed to start ffmpeg HLS: {e}")))?;

    sessions().lock().await.insert(
        session_id.clone(),
        HlsSession {
            dir,
            child: Some(child),
            last_access: Instant::now(),
        },
    );

    // Wait for the playlist to have at least one segment (up to 10s)
    for _ in 0..100 {
        if let Ok(content) = tokio::fs::read_to_string(&playlist_path).await {
            if content.contains("#EXTINF") {
                break;
            }
        }
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    }

    let url = format!("/cinema/api/hls/{session_id}/playlist.m3u8");
    Ok((session_id, url))
}

/// Touch the session's last_access timestamp.
pub async fn touch(session_id: &str) {
    if let Some(session) = sessions().lock().await.get_mut(session_id) {
        session.last_access = Instant::now();
    }
}

/// Get the directory for a session (if it exists).
pub async fn session_dir(session_id: &str) -> Option<PathBuf> {
    sessions().lock().await.get(session_id).map(|s| s.dir.clone())
}

/// Stop and clean up a specific session.
pub async fn stop_session(session_id: &str) {
    let session = sessions().lock().await.remove(session_id);
    if let Some(mut session) = session {
        if let Some(ref mut child) = session.child {
            let _ = child.kill().await;
        }
        let _ = tokio::fs::remove_dir_all(&session.dir).await;
    }
}

/// Clean up sessions that haven't been accessed in `max_idle_secs`.
/// Returns the number of sessions cleaned up.
pub async fn cleanup_idle(max_idle_secs: u64) -> usize {
    let mut map = sessions().lock().await;
    let now = Instant::now();
    let idle: Vec<String> = map
        .iter()
        .filter(|(_, s)| now.duration_since(s.last_access).as_secs() > max_idle_secs)
        .map(|(id, _)| id.clone())
        .collect();
    let count = idle.len();
    for id in idle {
        if let Some(mut session) = map.remove(&id) {
            if let Some(ref mut child) = session.child {
                let _ = child.kill().await;
            }
            let _ = tokio::fs::remove_dir_all(&session.dir).await;
        }
    }
    count
}

/// Stop all sessions (for shutdown).
pub async fn stop_all() {
    let mut map = sessions().lock().await;
    for (_, mut session) in map.drain() {
        if let Some(ref mut child) = session.child {
            let _ = child.kill().await;
        }
        let _ = tokio::fs::remove_dir_all(&session.dir).await;
    }
}
