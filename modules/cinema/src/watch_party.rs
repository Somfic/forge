use std::sync::{Arc, OnceLock};
use std::time::Instant;

use axum::extract::Path;
use axum::extract::ws::{WebSocket, WebSocketUpgrade};
use axum::response::IntoResponse;
use forge::ws::{RoomRegistry, WsRoom, run_ws_connection};
use futures::SinkExt;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;

// ── Message types ──

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ClientMsg {
    /// Host only — set the content for the room
    SetContent {
        media_type: String,
        tmdb_id: i64,
        title: String,
        poster_path: Option<String>,
        info_hash: String,
        file_idx: u32,
        season: Option<u32>,
        episode: Option<u32>,
    },
    /// Host only — sync navigation to guests
    Navigate { url: String },
    /// Stream has loaded and is ready to play
    Loaded,
    Ready,
    Unready,
    Play { position: f64 },
    Pause { position: f64 },
    Seek { position: f64 },
    Ping,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ServerMsg {
    /// Sent to host on room creation
    RoomCreated { code: String },
    /// Sent to the connecting participant with their assigned ID and role
    Welcome { id: u64, role: Role },
    /// Broadcast when someone joins
    ParticipantJoined { id: u64, count: usize },
    /// Broadcast when someone leaves
    ParticipantLeft { id: u64, count: usize },
    /// Room is closing (host left)
    RoomClosed { reason: String },
    /// Host has set the content
    ContentSet {
        media_type: String,
        tmdb_id: i64,
        title: String,
        poster_path: Option<String>,
        info_hash: String,
        file_idx: u32,
        season: Option<u32>,
        episode: Option<u32>,
    },
    /// Host navigated — guests should follow
    NavigateSync { url: String },
    /// Current ready state
    ReadyState { ready: Vec<u64>, total: usize },
    /// Playback sync
    Sync {
        playing: bool,
        position: f64,
        server_time: u64,
    },
    Pong,
    Error { message: String },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Role {
    Host,
    Guest,
}

// ── Room state ──

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Phase {
    Lobby,
    Picking,
    Ready,
    Watching,
}

#[derive(Debug, Clone)]
struct ContentPick {
    media_type: String,
    tmdb_id: i64,
    title: String,
    poster_path: Option<String>,
    info_hash: String,
    file_idx: u32,
    season: Option<u32>,
    episode: Option<u32>,
}

#[derive(Debug, Clone)]
struct PlaybackState {
    playing: bool,
    position: f64,
    updated_at: Instant,
}

struct ParticipantMeta {
    id: u64,
    role: Role,
    ready: bool,
    loaded: bool,
}

pub(crate) struct WatchPartyState {
    phase: Phase,
    host_id: u64,
    participant_meta: Vec<ParticipantMeta>,
    content: Option<ContentPick>,
    playback: Option<PlaybackState>,
}

// ── Registry ──

fn registry() -> &'static RoomRegistry<WatchPartyState> {
    static REGISTRY: OnceLock<RoomRegistry<WatchPartyState>> = OnceLock::new();
    REGISTRY.get_or_init(RoomRegistry::new)
}

// ── Handlers ──

pub async fn ws_create(ws: WebSocketUpgrade) -> impl IntoResponse {
    ws.on_upgrade(handle_create)
}

pub async fn ws_join(ws: WebSocketUpgrade, Path(code): Path<String>) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_join(socket, code.to_uppercase()))
}

async fn handle_create(socket: WebSocket) {
    let state = WatchPartyState {
        phase: Phase::Lobby,
        host_id: 0, // will be set below
        participant_meta: Vec::new(),
        content: None,
        playback: None,
    };

    let room = registry().create(state);
    let code = room.code.clone();

    let (id, rx) = room.add_participant();

    // Set host_id and add meta
    {
        let mut state = room.state.lock().unwrap();
        state.host_id = id;
        state.participant_meta.push(ParticipantMeta {
            id,
            role: Role::Host,
            ready: false,
            loaded: false,
        });
    }

    // Send room created + welcome
    room.send_to(id, &ServerMsg::RoomCreated { code: code.clone() });
    room.send_to(id, &ServerMsg::Welcome { id, role: Role::Host });

    run_participant(socket, rx, room, id).await;

    // Host left — destroy room
    cleanup_room(&code, id);
}

async fn handle_join(socket: WebSocket, code: String) {
    let room = match registry().get(&code) {
        Some(r) => r,
        None => {
            // Can't send error before upgrade, just close
            let (mut tx, _) = futures::StreamExt::split(socket);
            let _ = tx
                .send(axum::extract::ws::Message::text(
                    serde_json::to_string(&ServerMsg::Error {
                        message: "Room not found".into(),
                    })
                    .unwrap(),
                ))
                .await;
            return;
        }
    };

    let (id, rx) = room.add_participant();

    {
        let mut state = room.state.lock().unwrap();
        state.participant_meta.push(ParticipantMeta {
            id,
            role: Role::Guest,
            ready: false,
            loaded: false,
        });

        // Transition from Lobby to Picking when first guest joins
        if state.phase == Phase::Lobby {
            state.phase = Phase::Picking;
        }
    }

    // Send welcome to the new participant
    room.send_to(id, &ServerMsg::Welcome { id, role: Role::Guest });

    // If content already set, send it to the new participant
    {
        let state = room.state.lock().unwrap();
        if let Some(ref content) = state.content {
            room.send_to(
                id,
                &ServerMsg::ContentSet {
                    media_type: content.media_type.clone(),
                    tmdb_id: content.tmdb_id,
                    title: content.title.clone(),
                    poster_path: content.poster_path.clone(),
                    info_hash: content.info_hash.clone(),
                    file_idx: content.file_idx,
                    season: content.season,
                    episode: content.episode,
                },
            );
        }
    }

    // Broadcast join to everyone
    let count = room.participant_count();
    room.broadcast(&ServerMsg::ParticipantJoined { id, count });

    run_participant(socket, rx, room, id).await;

    // Guest left
    cleanup_participant(&code, id);
}

async fn run_participant(
    socket: WebSocket,
    rx: mpsc::UnboundedReceiver<String>,
    room: Arc<WsRoom<WatchPartyState>>,
    my_id: u64,
) {
    let room_clone = room.clone();

    run_ws_connection(socket, rx, move |msg: ClientMsg| {
        handle_message(&room_clone, my_id, msg);
        None::<ServerMsg>
    })
    .await;

    // Start heartbeat for sync during watching phase
    // (Handled in a separate task below)
}

fn handle_message(room: &Arc<WsRoom<WatchPartyState>>, sender_id: u64, msg: ClientMsg) {
    match msg {
        ClientMsg::SetContent {
            media_type,
            tmdb_id,
            title,
            poster_path,
            info_hash,
            file_idx,
            season,
            episode,
        } => {
            let mut state = room.state.lock().unwrap();

            // Only host can set content
            if state.host_id != sender_id {
                room.send_to(
                    sender_id,
                    &ServerMsg::Error {
                        message: "Only the host can set content".into(),
                    },
                );
                return;
            }

            state.content = Some(ContentPick {
                media_type: media_type.clone(),
                tmdb_id,
                title: title.clone(),
                poster_path: poster_path.clone(),
                info_hash: info_hash.clone(),
                file_idx,
                season,
                episode,
            });

            // Reset loaded flags for all participants
            for p in &mut state.participant_meta {
                p.loaded = false;
            }
            state.phase = Phase::Watching;
            state.playback = Some(PlaybackState {
                playing: false,
                position: 0.0,
                updated_at: Instant::now(),
            });

            drop(state);

            room.broadcast(&ServerMsg::ContentSet {
                media_type,
                tmdb_id,
                title,
                poster_path,
                info_hash,
                file_idx,
                season,
                episode,
            });
        }

        ClientMsg::Ready => {
            let mut state = room.state.lock().unwrap();
            if state.phase != Phase::Ready {
                return;
            }

            if let Some(p) = state.participant_meta.iter_mut().find(|p| p.id == sender_id) {
                p.ready = true;
            }

            let all_ready = state.participant_meta.iter().all(|p| p.ready);

            if all_ready && state.participant_meta.len() >= 2 {
                state.phase = Phase::Watching;
                state.playback = Some(PlaybackState {
                    playing: false,
                    position: 0.0,
                    updated_at: Instant::now(),
                });
            }

            drop(state);
            broadcast_ready_state(room);

            // If we just transitioned to watching, send initial sync
            let state = room.state.lock().unwrap();
            if state.phase == Phase::Watching {
                drop(state);
                broadcast_sync(room);
            }
        }

        ClientMsg::Unready => {
            let mut state = room.state.lock().unwrap();
            if state.phase != Phase::Ready {
                return;
            }

            if let Some(p) = state.participant_meta.iter_mut().find(|p| p.id == sender_id) {
                p.ready = false;
            }

            drop(state);
            broadcast_ready_state(room);
        }

        ClientMsg::Play { position } => {
            let mut state = room.state.lock().unwrap();
            if state.phase != Phase::Watching {
                return;
            }

            if let Some(ref mut pb) = state.playback {
                pb.playing = true;
                pb.position = position;
                pb.updated_at = Instant::now();
            }

            drop(state);
            broadcast_sync_except(room, sender_id);
        }

        ClientMsg::Pause { position } => {
            let mut state = room.state.lock().unwrap();
            if state.phase != Phase::Watching {
                return;
            }

            if let Some(ref mut pb) = state.playback {
                pb.playing = false;
                pb.position = position;
                pb.updated_at = Instant::now();
            }

            drop(state);
            broadcast_sync_except(room, sender_id);
        }

        ClientMsg::Seek { position } => {
            let mut state = room.state.lock().unwrap();
            if state.phase != Phase::Watching {
                return;
            }

            if let Some(ref mut pb) = state.playback {
                pb.position = position;
                pb.updated_at = Instant::now();
            }

            drop(state);
            broadcast_sync_except(room, sender_id);
        }

        ClientMsg::Loaded => {
            let mut state = room.state.lock().unwrap();
            if state.phase != Phase::Watching {
                return;
            }

            if let Some(p) = state.participant_meta.iter_mut().find(|p| p.id == sender_id) {
                p.loaded = true;
            }

            let all_loaded = state.participant_meta.iter().all(|p| p.loaded);
            if all_loaded {
                // Everyone's stream is ready — start playback
                if let Some(ref mut pb) = state.playback {
                    pb.playing = true;
                    pb.position = 0.0;
                    pb.updated_at = Instant::now();
                }
                drop(state);
                broadcast_sync(room);
            }
        }

        ClientMsg::Navigate { url } => {
            let state = room.state.lock().unwrap();
            if state.host_id != sender_id {
                return;
            }
            drop(state);
            room.broadcast_except(sender_id, &ServerMsg::NavigateSync { url });
        }

        ClientMsg::Ping => {
            room.send_to(sender_id, &ServerMsg::Pong);
        }
    }
}

fn broadcast_ready_state(room: &WsRoom<WatchPartyState>) {
    let state = room.state.lock().unwrap();
    let ready: Vec<u64> = state
        .participant_meta
        .iter()
        .filter(|p| p.ready)
        .map(|p| p.id)
        .collect();
    let total = state.participant_meta.len();
    drop(state);
    room.broadcast(&ServerMsg::ReadyState { ready, total });
}

fn make_sync_msg(room: &WsRoom<WatchPartyState>) -> Option<ServerMsg> {
    let state = room.state.lock().unwrap();
    let pb = state.playback.as_ref()?;
    let elapsed = if pb.playing {
        pb.updated_at.elapsed().as_secs_f64()
    } else {
        0.0
    };
    let playing = pb.playing;
    let position = pb.position + elapsed;
    let server_time = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64;
    Some(ServerMsg::Sync {
        playing,
        position,
        server_time,
    })
}

fn broadcast_sync(room: &WsRoom<WatchPartyState>) {
    if let Some(msg) = make_sync_msg(room) {
        room.broadcast(&msg);
    }
}

fn broadcast_sync_except(room: &WsRoom<WatchPartyState>, exclude_id: u64) {
    if let Some(msg) = make_sync_msg(room) {
        room.broadcast_except(exclude_id, &msg);
    }
}

fn cleanup_room(code: &str, _host_id: u64) {
    if let Some(room) = registry().remove(code) {
        room.broadcast(&ServerMsg::RoomClosed {
            reason: "Host left".into(),
        });
    }
}

fn cleanup_participant(code: &str, participant_id: u64) {
    if let Some(room) = registry().get(code) {
        room.remove_participant(participant_id);

        {
            let mut state = room.state.lock().unwrap();
            state.participant_meta.retain(|p| p.id != participant_id);
        }

        let count = room.participant_count();
        room.broadcast(&ServerMsg::ParticipantLeft {
            id: participant_id,
            count,
        });

        // Update ready state if in ready phase
        let state = room.state.lock().unwrap();
        if state.phase == Phase::Ready {
            drop(state);
            broadcast_ready_state(&room);
        }
    }
}

// ── Schema export ──

#[cfg(test)]
pub fn export_ws_schemas() {
    let client_schema = schemars::schema_for!(ClientMsg);
    let server_schema = schemars::schema_for!(ServerMsg);

    let client_json = serde_json::to_string_pretty(&client_schema).unwrap();
    let server_json = serde_json::to_string_pretty(&server_schema).unwrap();

    std::fs::write("frontend/ws-client-msg.schema.json", &client_json).unwrap();
    std::fs::write("frontend/ws-server-msg.schema.json", &server_json).unwrap();

    println!("Wrote WS schemas to frontend/ws-*.schema.json");
}
