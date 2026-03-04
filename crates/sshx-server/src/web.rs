//! HTTP and WebSocket handlers for the sshx web interface.

use std::sync::Arc;

use axum::body::Body;
use axum::extract::Request;
use axum::http::{header, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::any;
use axum::Router;
use include_dir::{include_dir, Dir};

use crate::ServerState;

pub mod protocol;
mod socket;

/// The SvelteKit static build, embedded at compile time.
/// Ensure `npm run build` has been run from the workspace root before compiling.
static BUILD_DIR: Dir<'static> = include_dir!("$CARGO_MANIFEST_DIR/../../build");

/// Returns the web application server, routed with Axum.
pub fn app() -> Router<Arc<ServerState>> {
    Router::new()
        .nest("/api", backend())
        .fallback(serve_static)
}

/// Routes for the backend web API server.
fn backend() -> Router<Arc<ServerState>> {
    Router::new().route("/s/{name}", any(socket::get_session_ws))
}

/// Serve an embedded static file with content-negotiation for precompressed variants.
///
/// Resolution order for a request path `P`:
/// 1. `P` with brotli encoding   (`P.br`)  if client accepts `br`
/// 2. `P` with gzip encoding     (`P.gz`)  if client accepts `gzip`
/// 3. `P` raw
/// 4. SPA fallback: `spa.html`   (same compression priority) for unknown paths
async fn serve_static(req: Request) -> Response {
    let path = req.uri().path().trim_start_matches('/');

    // Empty path → "index.html", which SvelteKit puts at root.
    let path = if path.is_empty() { "index.html" } else { path };

    // Detect which encodings the client accepts.
    let accept_enc = req
        .headers()
        .get(header::ACCEPT_ENCODING)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    let accept_br = accept_enc.contains("br");
    let accept_gz = accept_enc.contains("gzip");

    // Try to find and serve the file (with optional precompressed variant).
    if let Some(resp) = try_serve(path, accept_br, accept_gz) {
        return resp;
    }

    // SPA fallback: unknown paths are handled by the SvelteKit router client-side.
    if let Some(resp) = try_serve("spa.html", accept_br, accept_gz) {
        return resp;
    }

    (StatusCode::NOT_FOUND, "Not found").into_response()
}

/// Try to serve `path` from the embedded build dir, preferring compressed variants.
fn try_serve(path: &str, accept_br: bool, accept_gz: bool) -> Option<Response> {
    let content_type = mime_guess::from_path(path)
        .first()
        .map(|m| m.to_string())
        .unwrap_or_else(|| "application/octet-stream".to_string());

    // Cache-control: immutable for content-addressed _app/ assets, no-store for SPA HTML.
    let cache_control = if path.starts_with("_app/immutable/") {
        "public, max-age=31536000, immutable"
    } else if path.ends_with(".html") {
        "no-cache, no-store"
    } else {
        "public, max-age=3600"
    };

    // Brotli preferred.
    if accept_br {
        let br_path = format!("{path}.br");
        if let Some(file) = BUILD_DIR.get_file(&br_path) {
            return Some(build_response(
                file.contents(),
                &content_type,
                Some("br"),
                cache_control,
            ));
        }
    }

    // Gzip fallback.
    if accept_gz {
        let gz_path = format!("{path}.gz");
        if let Some(file) = BUILD_DIR.get_file(&gz_path) {
            return Some(build_response(
                file.contents(),
                &content_type,
                Some("gzip"),
                cache_control,
            ));
        }
    }

    // Plain file.
    BUILD_DIR.get_file(path).map(|file| {
        build_response(file.contents(), &content_type, None, cache_control)
    })
}

fn build_response(
    body: &'static [u8],
    content_type: &str,
    encoding: Option<&str>,
    cache_control: &str,
) -> Response {
    let mut builder = Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, content_type)
        .header(header::CACHE_CONTROL, cache_control);

    if let Some(enc) = encoding {
        builder = builder.header(header::CONTENT_ENCODING, enc);
    }

    builder.body(Body::from(body)).unwrap()
}
