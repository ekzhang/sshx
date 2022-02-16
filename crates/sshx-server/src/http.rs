//! HTTP and WebSocket handlers for the sshx web interface.

use axum::{body::Body, routing::get, Router};
use tower_http::trace::TraceLayer;

/// Returns the web application server, built with Axum.
pub fn app() -> Router<Body> {
    Router::new()
        .route("/", get(|| async { "Hello, world!" }))
        .layer(TraceLayer::new_for_http())
}
