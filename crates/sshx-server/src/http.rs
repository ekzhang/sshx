//! HTTP and WebSocket handlers for the sshx web interface.

use axum::{body::Body, routing::get, Router};

/// Returns the web application server, built with Axum.
pub fn app() -> Router<Body> {
    Router::new().route("/", get(|| async { "Hello, world!" }))
}
