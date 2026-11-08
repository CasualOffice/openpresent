//! The OpenPresent collaboration server — **a session, not a filing system**.
//!
//! Copied in part from OpenCalc's `casual-calc-collab-server` at `b23c67c`
//! (PROVENANCE.md): its transport (one WebSocket per participant, a first message
//! that must be a join, a frame limit set before a byte is read), its ordering (one
//! `ServerSession` per document, a chunk rebased over what was committed since its
//! base, acknowledged to its sender and relayed as it landed to everyone else), its
//! presence relay and its colours. Everything engine-specific goes through
//! `casual-slide-transaction::collab`'s [`Engine`] seam, so this crate compiles
//! against no model, reader or writer, and **never reads a deck**: the first
//! participant seeds a room with the deck's bytes, which are handed to each joiner
//! as they came, with the operations since.
//!
//! Not copied (docs/24 §The server, the audit): tokens and their verifier, the
//! integrator's document fetch and save-back, the Redis cluster, leases and relay
//! between nodes, the save lifecycle, resume, metrics and TLS. A room is reached by
//! an unguessable key in its link and lives while someone is in it.

#![forbid(unsafe_code)]

use std::collections::BTreeMap;
use std::net::SocketAddr;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use axum::Router;
use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::extract::{Query, State};
use axum::response::IntoResponse;
use axum::routing::get;
use base64::Engine as _;
use casual_slide_transaction::collab::wire::SlideEngine;
use casual_slide_transaction::collab::{
    ClientId, ClientMessage, Commit, Engine, MAX_FRAME_BYTES, ServerMessage, ServerSession,
};
use futures_util::{SinkExt, StreamExt};
use serde::Deserialize;
use tokio::sync::mpsc;

/// The engine this server orders edits for.
pub type Slides = SlideEngine;

/// What the server holds back on, all of it before any work is done.
#[derive(Debug, Clone, Copy)]
pub struct Limits {
    /// Rooms open at once.
    pub max_rooms: usize,
    /// Participants in one room.
    pub max_members: usize,
    /// Bytes of one message, either way.
    pub max_message_bytes: usize,
    /// Operations in one submission.
    pub max_ops: usize,
    /// How long a new connection may take to join.
    pub join_timeout: Duration,
    /// How long a participant may say nothing before it is let go.
    pub idle_timeout: Duration,
}

impl Default for Limits {
    fn default() -> Self {
        Self {
            max_rooms: 1000,
            max_members: 32,
            max_message_bytes: MAX_FRAME_BYTES,
            max_ops: 10_000,
            join_timeout: Duration::from_secs(10),
            idle_timeout: Duration::from_secs(60),
        }
    }
}

/// A stable colour for a participant, from OpenCalc's palette: deterministic in the
/// id, and clear of red, which reads as an error.
#[must_use]
pub fn colour_for(client: ClientId) -> String {
    const PALETTE: [&str; 8] = [
        "2F6DF6", "7C3AED", "0891B2", "C2410C", "9333EA", "0D9488", "B45309", "4F46E5",
    ];
    let i = usize::try_from(client.0 % PALETTE.len() as u64).unwrap_or(0);
    format!("#{}", PALETTE.get(i).copied().unwrap_or("2F6DF6"))
}

struct Member {
    name: String,
    colour: String,
    tx: mpsc::UnboundedSender<String>,
}

struct Room {
    /// The deck as seeded, base64, as it travels in a welcome.
    seed: String,
    session: ServerSession<Slides>,
    members: BTreeMap<ClientId, Member>,
    next: u64,
}

#[derive(Default)]
struct Rooms {
    rooms: BTreeMap<String, Room>,
}

/// The server's state.
pub struct Service {
    limits: Limits,
    rooms: Mutex<Rooms>,
}

impl core::fmt::Debug for Service {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Service")
            .field("limits", &self.limits)
            .finish_non_exhaustive()
    }
}

fn lock(m: &Mutex<Rooms>) -> std::sync::MutexGuard<'_, Rooms> {
    // A poisoned lock means a panic elsewhere; the rooms are still consistent,
    // since every change to them is one assignment.
    m.lock().unwrap_or_else(std::sync::PoisonError::into_inner)
}

/// Whether a room key is one this server accepts: 8 to 64 letters, digits and
/// hyphens, as a UUID is.
#[must_use]
pub fn valid_room(key: &str) -> bool {
    (8..=64).contains(&key.len()) && key.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'-')
}

fn encode(message: &ServerMessage<Slides>) -> String {
    serde_json::to_string(message).unwrap_or_else(|_| "{}".to_owned())
}

fn refusal(reason: &str) -> String {
    encode(&ServerMessage::Refused {
        reason: reason.to_owned(),
    })
}

fn stop(reason: &str) -> String {
    encode(&ServerMessage::Stopped {
        reason: reason.to_owned(),
    })
}

/// The router: `GET /collab?room=KEY` upgrades to the session; `GET /healthz`; and,
/// given a folder, the editor's files from it, so the page and its socket share an
/// origin and the page's `connect-src 'self'` holds (`COL-04`).
pub fn router(limits: Limits, files: Option<PathBuf>) -> Router {
    let state = Arc::new(Service {
        limits,
        rooms: Mutex::new(Rooms::default()),
    });
    let router = Router::new()
        .route("/collab", get(upgrade))
        .route("/healthz", get(|| async { "ok" }))
        .with_state(state);
    match files {
        Some(root) => router.fallback(move |uri: axum::http::Uri| {
            let root = root.clone();
            async move { static_file(&root, uri.path()).await }
        }),
        None => router,
    }
}

/// Serve on a bound listener until the process ends; `files`, a folder of the
/// editor's files to serve beside the session.
pub async fn serve(
    listener: tokio::net::TcpListener,
    limits: Limits,
    files: Option<PathBuf>,
) -> std::io::Result<()> {
    axum::serve(
        listener,
        router(limits, files).into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await
}

/// One file of the editor's folder. Only plain names in plain folders: no `..`, no
/// hidden file, nothing outside the folder; a type for what the editor ships.
async fn static_file(root: &Path, path: &str) -> axum::response::Response {
    use axum::http::{StatusCode, header};
    let path = path.trim_start_matches('/');
    let path = if path.is_empty() { "editor.html" } else { path };
    let plain = path.split('/').all(|seg| {
        !seg.is_empty()
            && !seg.starts_with('.')
            && seg
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'.' | b'_' | b'-'))
    });
    let kind = match path.rsplit('.').next().unwrap_or("") {
        "html" => "text/html; charset=utf-8",
        "js" => "text/javascript; charset=utf-8",
        "css" => "text/css; charset=utf-8",
        "wasm" => "application/wasm",
        "svg" => "image/svg+xml",
        "png" => "image/png",
        "json" => "application/json",
        "woff2" => "font/woff2",
        "ttf" => "font/ttf",
        "pptx" => "application/vnd.openxmlformats-officedocument.presentationml.presentation",
        _ => "",
    };
    if !plain || kind.is_empty() {
        return (StatusCode::NOT_FOUND, "not found").into_response();
    }
    match tokio::fs::read(root.join(path)).await {
        Ok(bytes) => (
            [
                (header::CONTENT_TYPE, kind),
                (header::CACHE_CONTROL, "no-cache"),
            ],
            bytes,
        )
            .into_response(),
        Err(_) => (StatusCode::NOT_FOUND, "not found").into_response(),
    }
}

#[derive(Deserialize)]
struct Join {
    room: String,
}

async fn upgrade(
    State(state): State<Arc<Service>>,
    Query(join): Query<Join>,
    ws: WebSocketUpgrade,
) -> impl IntoResponse {
    // Bounded before a byte is read (OpenCalc's rule).
    let ws = ws
        .max_message_size(state.limits.max_message_bytes)
        .max_frame_size(state.limits.max_message_bytes);
    ws.on_upgrade(move |socket| connection(state, join.room, socket))
}

/// Admit a participant: the join's checks, then a place in the room.
fn admit(
    state: &Service,
    room_key: &str,
    name: String,
    seed: Option<String>,
    tx: mpsc::UnboundedSender<String>,
) -> Result<(ClientId, String), &'static str> {
    let mut rooms = lock(&state.rooms);
    let open = rooms.rooms.len();
    if !rooms.rooms.contains_key(room_key) {
        if open >= state.limits.max_rooms {
            return Err("full");
        }
        let Some(seed) = seed else {
            return Err("noDocument");
        };
        // Checked as base64 and bounded; never read as a deck here.
        let bytes = base64::engine::general_purpose::STANDARD
            .decode(seed.as_bytes())
            .map_err(|_| "CS-COL-0004")?;
        if bytes.is_empty() {
            return Err("CS-COL-0004");
        }
        rooms.rooms.insert(
            room_key.to_owned(),
            Room {
                seed,
                session: ServerSession::new(),
                members: BTreeMap::new(),
                next: 0,
            },
        );
    }
    let room = rooms.rooms.get_mut(room_key).ok_or("gone")?;
    if room.members.len() >= state.limits.max_members {
        return Err("full");
    }
    room.next += 1;
    let client = ClientId(room.next);
    let colour = colour_for(client);
    let welcome = encode(&ServerMessage::Welcome {
        protocol: Slides::PROTOCOL,
        client,
        colour: colour.clone(),
        snapshot: room.seed.clone(),
        ops: room.session.history_since(0).unwrap_or_default().to_vec(),
        revision: room.session.revision(),
    });
    let _ = tx.send(welcome);
    // Everyone already here, so the newcomer sees them before they move.
    for (&id, m) in &room.members {
        let _ = tx.send(encode(&ServerMessage::Presence {
            client: id,
            name: m.name.clone(),
            colour: m.colour.clone(),
            presence: Default::default(),
        }));
    }
    let name: String = name.chars().filter(|c| !c.is_control()).take(64).collect();
    room.members.insert(
        client,
        Member {
            name,
            colour: colour.clone(),
            tx,
        },
    );
    Ok((client, colour))
}

fn leave(state: &Service, room_key: &str, client: ClientId) {
    let mut rooms = lock(&state.rooms);
    let empty = match rooms.rooms.get_mut(room_key) {
        Some(room) => {
            room.members.remove(&client);
            let gone = encode(&ServerMessage::Departed { client });
            for m in room.members.values() {
                let _ = m.tx.send(gone.clone());
            }
            room.members.is_empty()
        }
        None => false,
    };
    // A session, not a filing system: a room ends with its last participant.
    if empty {
        rooms.rooms.remove(room_key);
    }
}

/// Handle one message from a joined participant.
fn handle(state: &Service, room_key: &str, client: ClientId, text: &str) -> Option<String> {
    let message: ClientMessage<Slides> = match serde_json::from_str(text) {
        Ok(m) => m,
        Err(_) => return Some(refusal("CS-COL-0004")),
    };
    let mut rooms = lock(&state.rooms);
    let room = rooms.rooms.get_mut(room_key)?;
    match message {
        ClientMessage::Submit(mut submission) => {
            if submission.ops.len() > state.limits.max_ops {
                return Some(refusal("CS-COL-0004"));
            }
            // Who sent it is the connection's, never the message's say-so.
            submission.client = client;
            match room.session.commit(&submission) {
                Ok(Commit::Applied { ops, revision }) => {
                    let relay = encode(&ServerMessage::Apply { ops, revision });
                    for (&id, m) in &room.members {
                        if id != client {
                            let _ = m.tx.send(relay.clone());
                        }
                    }
                    Some(encode(&ServerMessage::Ack {
                        through: submission.seq,
                        revision,
                    }))
                }
                Ok(Commit::Duplicate { revision }) => Some(encode(&ServerMessage::Ack {
                    through: submission.seq,
                    revision,
                })),
                Err(e) => Some(stop(e.code())),
            }
        }
        ClientMessage::Presence(presence) => {
            let (name, colour) = room
                .members
                .get(&client)
                .map(|m| (m.name.clone(), m.colour.clone()))?;
            let relay = encode(&ServerMessage::Presence {
                client,
                name,
                colour,
                presence,
            });
            for (&id, m) in &room.members {
                if id != client {
                    let _ = m.tx.send(relay.clone());
                }
            }
            None
        }
        ClientMessage::Ping { nonce } => Some(encode(&ServerMessage::Pong { nonce })),
        ClientMessage::Heartbeat | ClientMessage::Leave => None,
        ClientMessage::Join { .. } => Some(refusal("joined")),
    }
}

async fn connection(state: Arc<Service>, room_key: String, socket: WebSocket) {
    let (mut sink, mut stream) = socket.split();
    if !valid_room(&room_key) {
        let _ = sink.send(Message::Text(stop("badRoom").into())).await;
        return;
    }
    // The first message must be a join, and soon (OpenCalc bounds an unjoined
    // connection in time the same way).
    let first = tokio::time::timeout(state.limits.join_timeout, stream.next()).await;
    let Ok(Some(Ok(Message::Text(first)))) = first else {
        return;
    };
    let Ok(ClientMessage::<Slides>::Join {
        protocol,
        name,
        seed,
    }) = serde_json::from_str(first.as_str())
    else {
        let _ = sink.send(Message::Text(stop("notJoined").into())).await;
        return;
    };
    if protocol != Slides::PROTOCOL {
        let _ = sink
            .send(Message::Text(stop("protocolVersion").into()))
            .await;
        return;
    }
    let (tx, mut rx) = mpsc::unbounded_channel::<String>();
    let (client, _) = match admit(&state, &room_key, name, seed, tx.clone()) {
        Ok(admitted) => admitted,
        Err(why) => {
            let _ = sink.send(Message::Text(stop(why).into())).await;
            return;
        }
    };
    let writer = tokio::spawn(async move {
        while let Some(text) = rx.recv().await {
            if sink.send(Message::Text(text.into())).await.is_err() {
                break;
            }
        }
    });
    loop {
        let next = tokio::time::timeout(state.limits.idle_timeout, stream.next()).await;
        let Ok(Some(Ok(message))) = next else { break };
        match message {
            Message::Text(text) => {
                if let Ok(ClientMessage::<Slides>::Leave) = serde_json::from_str(text.as_str()) {
                    break;
                }
                if let Some(reply) = handle(&state, &room_key, client, text.as_str()) {
                    let _ = tx.send(reply);
                }
            }
            Message::Close(_) => break,
            _ => {}
        }
    }
    leave(&state, &room_key, client);
    drop(tx);
    writer.abort();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn only_plain_files_of_the_editors_folder_are_served() {
        let root = std::env::temp_dir().join(format!("op-collab-files-{}", std::process::id()));
        std::fs::create_dir_all(root.join("pkg")).unwrap();
        std::fs::write(root.join("editor.html"), "<!doctype html>").unwrap();
        std::fs::write(root.join("pkg").join("a.wasm"), [0u8, 97, 115, 109]).unwrap();
        std::fs::write(root.join(".secret.js"), "x").unwrap();
        let status = |path: &'static str| {
            let root = root.clone();
            async move { static_file(&root, path).await.status().as_u16() }
        };
        assert_eq!(status("/").await, 200);
        assert_eq!(status("/pkg/a.wasm").await, 200);
        assert_eq!(status("/../Cargo.toml").await, 404);
        assert_eq!(status("/pkg/../editor.html").await, 404);
        assert_eq!(status("/.secret.js").await, 404);
        assert_eq!(status("/editor.exe").await, 404);
        assert_eq!(status("//etc/passwd").await, 404);
        let wasm = static_file(&root, "/pkg/a.wasm").await;
        assert_eq!(wasm.headers()["content-type"], "application/wasm");
        let _ = std::fs::remove_dir_all(&root);
    }
}
