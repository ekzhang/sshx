//! HTTP and WebSocket handlers for the sshx web interface.

use std::io;

use axum::routing::{get, get_service};
use axum::{extract::Path, Router};
use hyper::StatusCode;
use tower_http::services::{ServeDir, ServeFile};
use tracing::error;

use crate::session::SessionStore;

/// Returns the web application server, built with Axum.
pub fn app(store: SessionStore) -> Router {
    Router::new()
        .nest("/api", backend(store))
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
        .fallback(root_spa);

    Router::new().nest("/", get_service(static_files).handle_error(error_handler))
}

/// Runs the backend web API server.
fn backend(store: SessionStore) -> Router {
    let _ = store;
    Router::new().route(
        "/:message",
        get(|Path(message): Path<String>| async move { format!("got a message: {message}") }),
    )
}

/// Error handler for tower-http services.
async fn error_handler(error: io::Error) -> (StatusCode, String) {
    let message = format!("unhandled internal error: {error}");
    error!("{message}");
    (StatusCode::INTERNAL_SERVER_ERROR, message)
}
