use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};
use sshx_core::proto::{
    client_update::ClientMessage, server_update::ServerMessage, FileChunkResponse,
    FileDeleteRequest, FileDeletedResponse, FileDownloadRequest, FileEntry, FileListRequest,
    FileListResponse, FileRenameRequest, FileRenamedResponse, FileUploadRequest,
};
use tokio::sync::mpsc;

const CHUNK_SIZE: usize = 1 << 16; // 64 KiB

pub async fn handle_file_operation(
    msg: ServerMessage,
    root_dir: &Path,
    sender: &mpsc::Sender<ClientMessage>,
) -> Result<()> {
    match msg {
        ServerMessage::ListFiles(req) => list_files(req, root_dir, sender).await,
        ServerMessage::DownloadFile(req) => download_file(req, root_dir, sender).await,
        ServerMessage::UploadFile(req) => upload_file(req, root_dir, sender).await,
        ServerMessage::DeleteFile(req) => delete_file(req, root_dir, sender).await,
        ServerMessage::RenameFile(req) => rename_file(req, root_dir, sender).await,
        _ => bail!("unexpected message type for file operation"),
    }
}

fn resolve_path(root_dir: &Path, path: &str) -> Result<PathBuf> {
    let candidate = root_dir.join(path.trim_start_matches('/'));
    let root = root_dir.canonicalize().context("failed to resolve root")?;

    // Try full canonicalize; if the path doesn't exist, validate via parent
    let canonical = match candidate.canonicalize() {
        Ok(c) => c,
        Err(_) if !candidate.exists() => {
            let parent = candidate
                .parent()
                .context("path has no parent")?
                .canonicalize()
                .context("failed to resolve parent")?;
            let filename = candidate.file_name().context("path has no filename")?;
            parent.join(filename)
        }
        Err(_) => bail!("cannot access path: {}", path),
    };

    if !canonical.starts_with(&root) {
        bail!("path traversal attempt: {}", path);
    }
    Ok(canonical)
}

async fn list_files(
    req: FileListRequest,
    root_dir: &Path,
    sender: &mpsc::Sender<ClientMessage>,
) -> Result<()> {
    let dir = if req.path.is_empty() || req.path == "/" {
        root_dir.to_path_buf()
    } else {
        resolve_path(root_dir, &req.path)?
    };
    let mut entries = Vec::new();
    let mut read_dir = tokio::fs::read_dir(&dir).await?;
    while let Some(entry) = read_dir.next_entry().await? {
        let metadata = entry.metadata().await?;
        let name = entry.file_name().to_string_lossy().to_string();
        if name.starts_with('.') {
            continue;
        }
        entries.push(FileEntry {
            name,
            is_dir: metadata.is_dir(),
            size: metadata.len(),
            modified: metadata
                .modified()
                .ok()
                .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                .map(|d| d.as_millis() as i64)
                .unwrap_or(0),
        });
    }
    entries.sort_by(|a, b| a.name.cmp(&b.name));
    let _ = sender
        .send(ClientMessage::FileList(FileListResponse {
            id: req.id,
            entries,
            path: req.path,
        }))
        .await;
    Ok(())
}

async fn download_file(
    req: FileDownloadRequest,
    root_dir: &Path,
    sender: &mpsc::Sender<ClientMessage>,
) -> Result<()> {
    let path = resolve_path(root_dir, &req.path)?;
    if !path.is_file() {
        bail!("not a file: {}", req.path);
    }
    let size = path.metadata()?.len();
    let mut file = tokio::fs::File::open(&path).await?;
    let mut buf = vec![0u8; CHUNK_SIZE];
    use tokio::io::AsyncReadExt;
    loop {
        let n = file.read(&mut buf).await?;
        let done = n == 0;
        let data = if done { vec![] } else { buf[..n].to_vec() };
        if sender
            .send(ClientMessage::FileChunk(FileChunkResponse {
                id: req.id,
                data: data.into(),
                done,
                size,
            }))
            .await
            .is_err()
        {
            break;
        }
        if done {
            break;
        }
    }
    Ok(())
}

async fn upload_file(
    req: FileUploadRequest,
    root_dir: &Path,
    sender: &mpsc::Sender<ClientMessage>,
) -> Result<()> {
    let path = resolve_path(root_dir, &req.path)?;
    if path.is_dir() {
        bail!("cannot upload to a directory: {}", req.path);
    }
    if let Some(parent) = path.parent() {
        tokio::fs::create_dir_all(parent).await?;
    }
    use tokio::io::AsyncWriteExt;
    let mut file = tokio::fs::File::create(&path).await?;
    file.write_all(&req.data).await?;
    if req.done {
        file.flush().await?;
        drop(file);
    }
    if req.done {
        let _ = sender
            .send(ClientMessage::FileChunk(FileChunkResponse {
                id: req.id,
                data: vec![].into(),
                done: true,
                size: 0,
            }))
            .await;
    }
    Ok(())
}

async fn delete_file(
    req: FileDeleteRequest,
    root_dir: &Path,
    sender: &mpsc::Sender<ClientMessage>,
) -> Result<()> {
    let path = resolve_path(root_dir, &req.path)?;
    if path == root_dir.canonicalize()? {
        bail!("cannot delete root directory");
    }
    let result = if path.is_dir() {
        tokio::fs::remove_dir_all(&path).await
    } else {
        tokio::fs::remove_file(&path).await
    };
    let error = result.err().map(|e| e.to_string()).unwrap_or_default();
    let path = req.path.clone();
    let _ = sender
        .send(ClientMessage::FileDeleted(FileDeletedResponse {
            id: req.id,
            error,
            path,
        }))
        .await;
    Ok(())
}

async fn rename_file(
    req: FileRenameRequest,
    root_dir: &Path,
    sender: &mpsc::Sender<ClientMessage>,
) -> Result<()> {
    let old_path = resolve_path(root_dir, &req.path)?;
    let new_path = resolve_path(root_dir, &req.new_name)?;
    let result = tokio::fs::rename(&old_path, &new_path).await;
    let (np, error) = match result {
        Ok(()) => (req.new_name.clone(), String::new()),
        Err(e) => (String::new(), e.to_string()),
    };
    let _ = sender
        .send(ClientMessage::FileRenamed(FileRenamedResponse {
            id: req.id,
            new_path: np,
            error,
        }))
        .await;
    Ok(())
}
