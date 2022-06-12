//! HTTP and WebSocket handlers for the sshx web interface.

use std::io;
use std::sync::Arc;

use anyhow::Result;
use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::extract::Path;
use axum::middleware::{self, Next};
use axum::response::{IntoResponse, Redirect, Response};
use axum::routing::{get, get_service};
use axum::{Extension, Router};
use hyper::{Request, StatusCode};
use serde::{Deserialize, Serialize};
use tower_http::services::{ServeDir, ServeFile};
use tracing::{error, info, info_span, warn, Instrument};

use crate::session::Session;
use crate::state::ServerState;

/// Returns the web application server, built with Axum.
pub fn app(state: Arc<ServerState>) -> Router {
    Router::new()
        .nest("/api", backend(state))
        .fallback(frontend())
}

/// Serves static SvelteKit build files.
fn frontend() -> Router {
    let root_spa = ServeFile::new("build/spa.html")
        .precompressed_gzip()
        .precompressed_br();

    let static_files = ServeDir::new("build")
        .precompressed_gzip()
        .precompressed_br()
        .not_found_service(root_spa);

    // Remove unnecessary trailing "/index.html" or ".html" from the static file
    // path, if it exists. Also translates 404 codes into 200.
    let remove_dot_html = middleware::from_fn(|req: Request<_>, next: Next<_>| async {
        let uri = req.uri().clone();
        let mut resp = next.run(req).await;
        if resp.status().is_success() {
            let path = uri.path();
            if let Some(new_path) = path
                .strip_suffix("/index.html")
                .or_else(|| path.strip_suffix(".html"))
            {
                let mut location = String::from(new_path);
                if location.is_empty() {
                    location += "/";
                }
                if let Some(query) = uri.query() {
                    location += "?";
                    location += query;
                }
                return Err(Redirect::temporary(&location));
            }
        } else if resp.status() == StatusCode::NOT_FOUND {
            *resp.status_mut() = StatusCode::OK;
        }
        Ok(resp)
    });

    Router::new()
        .nest("/", get_service(static_files).handle_error(error_handler))
        .layer(remove_dot_html)
}

/// Error handler for tower-http services.
async fn error_handler(error: io::Error) -> impl IntoResponse {
    let message = format!("unhandled internal error: {error}");
    error!("{message}");
    (StatusCode::INTERNAL_SERVER_ERROR, message)
}

/// Runs the backend web API server.
fn backend(state: Arc<ServerState>) -> Router {
    Router::new()
        .route("/s/:id", get(get_session_ws))
        .layer(Extension(state))
}

/// A real-time message sent from the server over WebSocket.
#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub enum WsServer {
    /// Notification when the set of open shells has changed.
    Shells(Vec<u32>),
    /// Subscription results, chunks of terminal data.
    Chunks(u32, Vec<(u64, String)>),
}

/// A real-time message sent from the client over WebSocket.
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub enum WsClient {
    /// Create a new shell.
    Create,
    /// Close a specific shell.
    Close(u32),
    /// Add user data to a given shell.
    Data(u32, String),
    /// Subscribe to a shell, starting at a given chunk index.
    Subscribe(u32, u64),
}

async fn get_session_ws(
    Path(id): Path<String>,
    ws: WebSocketUpgrade,
    Extension(state): Extension<Arc<ServerState>>,
) -> Response {
    if let Some(session) = state.store.get(&id) {
        let session = Arc::clone(&*session);
        ws.on_upgrade(move |socket| {
            async {
                if let Err(err) = handle_socket(socket, session).await {
                    warn!(?err, "exiting early");
                }
            }
            .instrument(info_span!("ws", %id))
        })
    } else {
        (StatusCode::NOT_FOUND, "session not found").into_response()
    }
}

/// Handle an incoming live WebSocket connection to a given session.
async fn handle_socket(mut socket: WebSocket, session: Arc<Session>) -> Result<()> {
    /// Send a message to the client over WebSocket.
    async fn send(socket: &mut WebSocket, msg: WsServer) -> Result<()> {
        let msg = serde_json::to_string(&msg)?;
        socket.send(Message::Text(msg)).await?;
        Ok(())
    }

    /// Receive a message from the client over WebSocket.
    async fn recv(socket: &mut WebSocket) -> Result<Option<WsClient>> {
        Ok(loop {
            match socket.recv().await.transpose()? {
                Some(Message::Text(msg)) => break Some(serde_json::from_str(&msg)?),
                Some(Message::Binary(_)) => warn!("ignoring binary message over WebSocket"),
                Some(_) => (), // ignore other message types, keep looping
                None => break None,
            }
        })
    }

    let _ = session;
    send(&mut socket, WsServer::Shells(vec![])).await?;
    let msg = recv(&mut socket).await?;
    info!(?msg);
    Ok(())
}
