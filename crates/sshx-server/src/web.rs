//! HTTP and WebSocket handlers for the sshx web interface.

use std::io;
use std::sync::Arc;

use axum::extract::ws::{WebSocket, WebSocketUpgrade};
use axum::extract::Path;
use axum::middleware::{self, Next};
use axum::response::{IntoResponse, Redirect, Response};
use axum::routing::{get, get_service};
use axum::{Extension, Router};
use hyper::{Request, StatusCode};
use tower_http::services::{ServeDir, ServeFile};
use tracing::{error, info_span, Instrument};

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

async fn get_session_ws(
    Path(id): Path<String>,
    ws: WebSocketUpgrade,
    Extension(state): Extension<Arc<ServerState>>,
) -> Response {
    if let Some(session) = state.store.get(&id) {
        let session = Arc::clone(&*session);
        ws.on_upgrade(move |socket| {
            handle_socket(socket, session).instrument(info_span!("ws", %id))
        })
    } else {
        (StatusCode::NOT_FOUND, "session not found").into_response()
    }
}

async fn handle_socket(mut _socket: WebSocket, _session: Arc<Session>) {
    todo!()
}
