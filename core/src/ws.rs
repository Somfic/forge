use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use axum::extract::ws::{Message, WebSocket};
use futures::{SinkExt, StreamExt};
use rand::Rng;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;

/// A participant in a WebSocket room.
pub struct RoomParticipant {
    pub id: u64,
    tx: mpsc::UnboundedSender<String>,
}

impl RoomParticipant {
    pub fn send(&self, msg: &impl Serialize) -> bool {
        match serde_json::to_string(msg) {
            Ok(json) => self.tx.send(json).is_ok(),
            Err(_) => false,
        }
    }

    pub fn is_connected(&self) -> bool {
        !self.tx.is_closed()
    }
}

/// A generic WebSocket room with custom state and N participants.
pub struct WsRoom<S: Send> {
    pub code: String,
    pub state: Mutex<S>,
    participants: Mutex<Vec<RoomParticipant>>,
    next_id: Mutex<u64>,
}

impl<S: Send> WsRoom<S> {
    pub fn new(code: String, state: S) -> Self {
        Self {
            code,
            state: Mutex::new(state),
            participants: Mutex::new(Vec::new()),
            next_id: Mutex::new(1),
        }
    }

    /// Add a participant and return (id, receiver for outgoing messages).
    pub fn add_participant(&self) -> (u64, mpsc::UnboundedReceiver<String>) {
        let (tx, rx) = mpsc::unbounded_channel();
        let mut next_id = self.next_id.lock().unwrap();
        let id = *next_id;
        *next_id += 1;
        self.participants.lock().unwrap().push(RoomParticipant { id, tx });
        (id, rx)
    }

    /// Remove a participant by ID. Returns true if found.
    pub fn remove_participant(&self, id: u64) -> bool {
        let mut participants = self.participants.lock().unwrap();
        let len_before = participants.len();
        participants.retain(|p| p.id != id);
        participants.len() < len_before
    }

    /// Broadcast a message to all participants.
    pub fn broadcast(&self, msg: &impl Serialize) {
        let json = match serde_json::to_string(msg) {
            Ok(j) => j,
            Err(_) => return,
        };
        let participants = self.participants.lock().unwrap();
        for p in participants.iter() {
            let _ = p.tx.send(json.clone());
        }
    }

    /// Broadcast a message to all participants except one.
    pub fn broadcast_except(&self, exclude_id: u64, msg: &impl Serialize) {
        let json = match serde_json::to_string(msg) {
            Ok(j) => j,
            Err(_) => return,
        };
        let participants = self.participants.lock().unwrap();
        for p in participants.iter() {
            if p.id != exclude_id {
                let _ = p.tx.send(json.clone());
            }
        }
    }

    /// Send a message to a specific participant.
    pub fn send_to(&self, id: u64, msg: &impl Serialize) -> bool {
        let participants = self.participants.lock().unwrap();
        if let Some(p) = participants.iter().find(|p| p.id == id) {
            p.send(msg)
        } else {
            false
        }
    }

    pub fn participant_count(&self) -> usize {
        self.participants.lock().unwrap().len()
    }

    pub fn participant_ids(&self) -> Vec<u64> {
        self.participants.lock().unwrap().iter().map(|p| p.id).collect()
    }
}

/// Registry of rooms, keyed by room code.
pub struct RoomRegistry<S: Send> {
    rooms: Mutex<HashMap<String, Arc<WsRoom<S>>>>,
}

impl<S: Send> RoomRegistry<S> {
    pub fn new() -> Self {
        Self {
            rooms: Mutex::new(HashMap::new()),
        }
    }

    /// Create a new room with a random 6-character code.
    pub fn create(&self, state: S) -> Arc<WsRoom<S>> {
        let code = generate_room_code();
        let room = Arc::new(WsRoom::new(code.clone(), state));
        self.rooms.lock().unwrap().insert(code, room.clone());
        room
    }

    /// Look up a room by code.
    pub fn get(&self, code: &str) -> Option<Arc<WsRoom<S>>> {
        self.rooms.lock().unwrap().get(code).cloned()
    }

    /// Remove a room by code.
    pub fn remove(&self, code: &str) -> Option<Arc<WsRoom<S>>> {
        self.rooms.lock().unwrap().remove(code)
    }

    /// Remove rooms that match a predicate.
    pub fn retain(&self, f: impl Fn(&str, &Arc<WsRoom<S>>) -> bool) {
        self.rooms.lock().unwrap().retain(|k, v| f(k, v));
    }

    pub fn count(&self) -> usize {
        self.rooms.lock().unwrap().len()
    }
}

/// Split a WebSocket into a JSON read/write pair tied to a room participant.
///
/// Returns the join handle for the read loop. The write loop is spawned internally.
/// When the read loop ends (client disconnects), the write loop is also dropped.
pub async fn run_ws_connection<R, W>(
    socket: WebSocket,
    mut outgoing_rx: mpsc::UnboundedReceiver<String>,
    mut on_message: impl FnMut(R) -> Option<W> + Send + 'static,
) where
    R: for<'de> Deserialize<'de> + Send + 'static,
    W: Serialize + Send + 'static,
{
    let (mut ws_tx, mut ws_rx) = socket.split();

    // Write loop: forward outgoing messages to the WebSocket
    let write_handle = tokio::spawn(async move {
        while let Some(json) = outgoing_rx.recv().await {
            if ws_tx.send(Message::text(json)).await.is_err() {
                break;
            }
        }
    });

    // Read loop: parse incoming JSON messages
    while let Some(Ok(msg)) = ws_rx.next().await {
        match msg {
            Message::Text(text) => {
                if let Ok(parsed) = serde_json::from_str::<R>(&text) {
                    if let Some(response) = on_message(parsed) {
                        // Response is sent via room broadcast, not directly
                        // But if we need direct response, we could add a channel
                        let _ = response; // handled by on_message callback
                    }
                }
            }
            Message::Close(_) => break,
            _ => {}
        }
    }

    write_handle.abort();
}

fn generate_room_code() -> String {
    let mut rng = rand::thread_rng();
    let chars: Vec<char> = "ABCDEFGHJKLMNPQRSTUVWXYZ23456789".chars().collect();
    (0..6).map(|_| chars[rng.gen_range(0..chars.len())]).collect()
}
