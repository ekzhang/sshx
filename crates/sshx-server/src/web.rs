//! HTTP and WebSocket handlers for the sshx web interface.

use std::collections::HashSet;
use std::io;
use std::sync::Arc;

use anyhow::{Context, Result};
use axum::extract::ws::{CloseFrame, Message, WebSocket, WebSocketUpgrade};
use axum::extract::{Path, State};
use axum::response::IntoResponse;
use axum::routing::{get, get_service};
use axum::Router;
use hyper::StatusCode;
use rand::Rng;
use serde::{Deserialize, Serialize};
use sshx_core::proto::{server_update::ServerMessage, TerminalInput, TerminalSize};
use sshx_core::{Sid, Uid};
use tokio::sync::mpsc;
use tokio_stream::StreamExt;
use tower_http::services::{ServeDir, ServeFile};
use tracing::{error, info_span, warn, Instrument};

use crate::session::Session;
use crate::state::ServerState;

/// Returns the web application server, built with Axum.
pub fn app() -> Router<Arc<ServerState>> {
    let root_spa = ServeFile::new("build/spa.html")
        .precompressed_gzip()
        .precompressed_br();

    // Serves static SvelteKit build files.
    let static_files = ServeDir::new("build")
        .precompressed_gzip()
        .precompressed_br()
        .fallback(root_spa);

    Router::new()
        .nest("/api", backend())
        .fallback_service(get_service(static_files).handle_error(error_handler))
}

/// Error handler for tower-http services.
async fn error_handler(error: io::Error) -> impl IntoResponse {
    let message = format!("unhandled internal error: {error}");
    error!("{message}");
    (StatusCode::INTERNAL_SERVER_ERROR, message)
}

/// Runs the backend web API server.
fn backend() -> Router<Arc<ServerState>> {
    Router::new().route("/s/:id", get(get_session_ws))
}

/// Real-time message conveying the position and size of a terminal.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct WsWinsize {
    /// The top-left x-coordinate of the window, offset from origin.
    pub x: i32,
    /// The top-left y-coordinate of the window, offset from origin.
    pub y: i32,
    /// The number of rows in the window.
    pub rows: u16,
    /// The number of columns in the terminal.
    pub cols: u16,
}

impl Default for WsWinsize {
    fn default() -> Self {
        WsWinsize {
            x: 0,
            y: 0,
            rows: 24,
            cols: 80,
        }
    }
}

impl WsWinsize {
    /// Create a new window with default size and random position in a range.
    pub fn new_random() -> Self {
        let x = rand::thread_rng().gen_range(-50..=50);
        let y = rand::thread_rng().gen_range(-30..=30);
        Self {
            x,
            y,
            ..Default::default()
        }
    }
}

/// Real-time message providing information about a user.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct WsUser {
    /// The user's display name.
    pub name: String,
    /// Live coordinates of the mouse cursor, if available.
    pub cursor: Option<(i32, i32)>,
    /// Currently focused terminal window ID.
    pub focus: Option<Sid>,
}

/// A real-time message sent from the server over WebSocket.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub enum WsServer {
    /// Initial server message, informing the user of their ID.
    Hello(Uid),
    /// A snapshot of all current users in the session.
    Users(Vec<(Uid, WsUser)>),
    /// Info about a single user in the session: joined, left, or changed.
    UserDiff(Uid, Option<WsUser>),
    /// Notification when the set of open shells has changed.
    Shells(Vec<(Sid, WsWinsize)>),
    /// Subscription results, in the form of terminal data chunks.
    Chunks(Sid, Vec<(u64, String)>),
    /// The current session has been terminated.
    Terminated(),
    /// Alert the client of an application error.
    Error(String),
}

/// A real-time message sent from the client over WebSocket.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub enum WsClient {
    /// Set the name of the current user.
    SetName(String),
    /// Send real-time information about the user's cursor.
    SetCursor(Option<(i32, i32)>),
    /// Set the currently focused shell.
    SetFocus(Option<Sid>),
    /// Create a new shell.
    Create(),
    /// Close a specific shell.
    Close(Sid),
    /// Move a shell window to a new position and focus it.
    Move(Sid, Option<WsWinsize>),
    /// Add user data to a given shell.
    Data(Sid, #[serde(with = "serde_bytes")] Vec<u8>),
    /// Subscribe to a shell, starting at a given chunk index.
    Subscribe(Sid, u64),
}

async fn get_session_ws(
    Path(id): Path<String>,
    ws: WebSocketUpgrade,
    State(state): State<Arc<ServerState>>,
) -> impl IntoResponse {
    if let Some(session) = state.store.get(&id) {
        let session = Arc::clone(&*session);
        ws.on_upgrade(move |socket| {
            async {
                if let Err(err) = handle_socket(socket, session).await {
                    warn!(%err, "websocket exiting early");
                }
            }
            .instrument(info_span!("ws", %id))
        })
    } else {
        ws.on_upgrade(|mut socket| async move {
            let frame = CloseFrame {
                code: 4404,
                reason: "could not find the requested session".into(),
            };
            socket.send(Message::Close(Some(frame))).await.ok();
        })
    }
}

/// Handle an incoming live WebSocket connection to a given session.
async fn handle_socket(mut socket: WebSocket, session: Arc<Session>) -> Result<()> {
    /// Send a message to the client over WebSocket.
    async fn send(socket: &mut WebSocket, msg: WsServer) -> Result<()> {
        let mut buf = Vec::new();
        ciborium::ser::into_writer(&msg, &mut buf)?;
        socket.send(Message::Binary(buf)).await?;
        Ok(())
    }

    /// Receive a message from the client over WebSocket.
    async fn recv(socket: &mut WebSocket) -> Result<Option<WsClient>> {
        Ok(loop {
            match socket.recv().await.transpose()? {
                Some(Message::Text(_)) => warn!("ignoring text message over WebSocket"),
                Some(Message::Binary(msg)) => break Some(ciborium::de::from_reader(&*msg)?),
                Some(_) => (), // ignore other message types, keep looping
                None => break None,
            }
        })
    }

    let user_id = session.counter().next_uid();
    send(&mut socket, WsServer::Hello(user_id)).await?;

    let _user_guard = session.user_scope(user_id)?;

    let update_tx = session.update_tx(); // start listening for updates before any state reads
    let mut broadcast_stream = session.subscribe_broadcast();
    send(&mut socket, WsServer::Users(session.list_users())).await?;

    let mut subscribed = HashSet::new(); // prevent duplicate subscriptions
    let (chunks_tx, mut chunks_rx) = mpsc::channel::<(Sid, Vec<(u64, String)>)>(1);

    let mut shells_stream = session.subscribe_shells();
    loop {
        let msg = tokio::select! {
            _ = session.terminated() => {
                send(&mut socket, WsServer::Terminated()).await?;
                socket.close().await?;
                break;
            }
            Some(result) = broadcast_stream.next() => {
                let msg = result.context("client fell behind on broadcast stream")?;
                send(&mut socket, msg).await?;
                continue;
            }
            Some(shells) = shells_stream.next() => {
                send(&mut socket, WsServer::Shells(shells)).await?;
                continue;
            }
            Some((id, chunks)) = chunks_rx.recv() => {
                send(&mut socket, WsServer::Chunks(id, chunks)).await?;
                continue;
            }
            result = recv(&mut socket) => {
                match result? {
                    Some(msg) => msg,
                    None => break,
                }
            }
        };

        match msg {
            WsClient::SetName(name) => {
                session.update_user(user_id, |user| user.name = name)?;
            }
            WsClient::SetCursor(cursor) => {
                session.update_user(user_id, |user| user.cursor = cursor)?;
            }
            WsClient::SetFocus(id) => {
                session.update_user(user_id, |user| user.focus = id)?;
            }
            WsClient::Create() => {
                let id = session.counter().next_sid();
                update_tx.send(ServerMessage::CreateShell(id.0)).await?;
            }
            WsClient::Close(id) => {
                update_tx.send(ServerMessage::CloseShell(id.0)).await?;
            }
            WsClient::Move(id, winsize) => {
                if let Err(err) = session.move_shell(id, winsize) {
                    send(&mut socket, WsServer::Error(err.to_string())).await?;
                    continue;
                }
                if let Some(winsize) = winsize {
                    let msg = ServerMessage::Resize(TerminalSize {
                        id: id.0,
                        rows: winsize.rows as u32,
                        cols: winsize.cols as u32,
                    });
                    session.update_tx().send(msg).await?;
                }
            }
            WsClient::Data(id, data) => {
                let data = TerminalInput { id: id.0, data };
                update_tx.send(ServerMessage::Input(data)).await?;
            }
            WsClient::Subscribe(id, chunknum) => {
                if subscribed.contains(&id) {
                    continue;
                }
                subscribed.insert(id);
                let session = Arc::clone(&session);
                let chunks_tx = chunks_tx.clone();
                tokio::spawn(async move {
                    let stream = session.subscribe_chunks(id, chunknum);
                    tokio::pin!(stream);
                    while let Some(chunks) = stream.next().await {
                        if chunks_tx.send((id, chunks)).await.is_err() {
                            break;
                        }
                    }
                });
            }
        }
    }
    Ok(())
}
