use std::collections::HashSet;
use std::sync::Arc;

use anyhow::{Context, Result};
use axum::extract::{
    ws::{CloseFrame, Message, WebSocket, WebSocketUpgrade},
    Path, State,
};
use axum::response::IntoResponse;
use sshx_core::proto::{server_update::ServerMessage, TerminalInput, TerminalSize};
use sshx_core::Sid;
use tokio::sync::mpsc;
use tokio_stream::StreamExt;
use tracing::{info_span, warn, Instrument};

use crate::session::Session;
use crate::web::protocol::{WsClient, WsServer};
use crate::ServerState;

pub async fn get_session_ws(
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
    let (chunks_tx, mut chunks_rx) = mpsc::channel::<(Sid, Vec<Arc<str>>)>(1);

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
            WsClient::Chat(msg) => {
                session.send_chat(user_id, &msg)?;
            }
        }
    }
    Ok(())
}
