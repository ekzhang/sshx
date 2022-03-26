//! HTTP and WebSocket handlers for the sshx web interface.

use std::io;

use axum::{
    body::Body,
    extract::Path,
    routing::{get, get_service},
    Router,
};
use hyper::{Request, StatusCode};
use tower::{service_fn, Service};
use tower_http::services::{ServeDir, ServeFile};
use tracing::error;

use crate::session::SessionStore;

/// Returns the web application server, built with Axum.
pub fn app(store: SessionStore) -> Router<Body> {
    Router::new()
        .nest("/api", backend(store))
        .fallback(frontend())
}

/// Serves static SvelteKit build files.
fn frontend() -> Router<Body> {
    let service = service_fn(|req| async {
        let resp = ServeDir::new("build")
            .precompressed_gzip()
            .precompressed_br()
            .call(req)
            .await?;

        if resp.status() == 404 {
            ServeFile::new("build/spa.html")
                .precompressed_gzip()
                .precompressed_br()
                .call(Request::<Body>::default())
                .await
        } else {
            Ok(resp)
        }
    });

    Router::new().nest("/", get_service(service).handle_error(error_handler))
}

/// Runs the backend web API server.
fn backend(store: SessionStore) -> Router<Body> {
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
