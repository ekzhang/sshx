//! HTTP file transfer endpoints for downloading and uploading files
//! through a session's CLI connection.

use std::sync::Arc;

use axum::body::Body;
use axum::extract::{Path, Query, State};
use axum::http::{header, HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use base64::prelude::{Engine as _, BASE64_STANDARD};
use bytes::Bytes;
use serde::Deserialize;
use subtle::ConstantTimeEq;

use sshx_core::proto::server_update::ServerMessage;
use sshx_core::proto::{FileDownloadRequest, FileUploadRequest};

use crate::ServerState;

/// Query parameters for file transfer endpoints.
#[derive(Deserialize)]
pub struct FileParams {
    path: String,
}

/// Validates the session authentication token from the `X-SSHX-Key` header.
async fn validate_auth(
    state: &ServerState,
    name: &str,
    headers: &HeaderMap,
) -> Result<Arc<crate::session::Session>, StatusCode> {
    let session = state.lookup(name).ok_or(StatusCode::NOT_FOUND)?;

    let key = headers
        .get("X-SSHX-Key")
        .and_then(|v| v.to_str().ok())
        .ok_or(StatusCode::UNAUTHORIZED)?;

    let key_bytes = BASE64_STANDARD
        .decode(key)
        .map_err(|_| StatusCode::UNAUTHORIZED)?;

    if !bool::from(session.metadata().encrypted_zeros.ct_eq(&key_bytes)) {
        return Err(StatusCode::UNAUTHORIZED);
    }

    Ok(session)
}

/// HTTP GET handler that streams a file from the CLI via gRPC.
pub async fn download_file(
    State(state): State<Arc<ServerState>>,
    Path(name): Path<String>,
    Query(params): Query<FileParams>,
    headers: HeaderMap,
) -> Response {
    let session = match validate_auth(&state, &name, &headers).await {
        Ok(s) => s,
        Err(code) => return (code, "Unauthorized").into_response(),
    };

    if params.path.is_empty() {
        return (StatusCode::BAD_REQUEST, "path is required").into_response();
    }

    let file_id = session.counter().next_sid().0;
    let filename = std::path::Path::new(&params.path)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("download");

    let (chunk_tx, chunk_rx) = tokio::sync::mpsc::channel::<Bytes>(32);
    session.register_download(file_id, chunk_tx);
    let update_tx = session.update_tx().clone();
    let path = params.path.clone();
    tokio::spawn(async move {
        let msg = ServerMessage::DownloadFile(FileDownloadRequest { id: file_id, path });
        let _ = update_tx.send(msg).await;
    });

    let stream = async_stream::stream! {
        let mut rx = chunk_rx;
        while let Some(chunk) = rx.recv().await {
            if chunk.is_empty() {
                break;
            }
            yield Ok::<Bytes, std::convert::Infallible>(chunk);
        }
    };

    Response::builder()
        .header(header::CONTENT_TYPE, "application/octet-stream")
        .header(
            header::CONTENT_DISPOSITION,
            format!("attachment; filename=\"{}\"", filename),
        )
        .body(Body::from_stream(stream))
        .unwrap()
}

/// HTTP POST handler that uploads a file to the CLI via gRPC.
pub async fn upload_file(
    State(state): State<Arc<ServerState>>,
    Path(name): Path<String>,
    Query(params): Query<FileParams>,
    headers: HeaderMap,
    body: Bytes,
) -> Response {
    let session = match validate_auth(&state, &name, &headers).await {
        Ok(s) => s,
        Err(code) => return (code, "Unauthorized").into_response(),
    };

    if params.path.is_empty() {
        return (StatusCode::BAD_REQUEST, "path is required").into_response();
    }

    let file_id = session.counter().next_sid().0;
    let update_tx = session.update_tx().clone();
    const CHUNK_SIZE: usize = 1 << 16; // 64 KiB
    let path = params.path.clone();

    tokio::spawn(async move {
        let total = body.len();
        for (i, chunk) in body.chunks(CHUNK_SIZE).enumerate() {
            let is_last = i * CHUNK_SIZE + chunk.len() >= total;
            let msg = ServerMessage::UploadFile(FileUploadRequest {
                id: file_id,
                path: path.clone(),
                data: chunk.to_vec().into(),
                done: is_last,
            });
            if update_tx.send(msg).await.is_err() {
                break;
            }
        }
    });

    StatusCode::OK.into_response()
}
