use std::process::Stdio;
use std::time::Duration;

use anyhow::{anyhow, Context, Result};
use sshx_server::{Server, ServerOptions};
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::net::TcpListener;
use tokio::process::{Child, Command};
use tokio::sync::{mpsc, oneshot};
use tokio::task::JoinHandle;
use tokio::time::timeout;
use tracing::{info, warn};

/// A guard that manages the lifetime of the local server and cloudflared tunnel.
pub struct TunnelGuard {
    /// The unencrypted HTTP local endpoint bounding the server.
    pub local_endpoint: String,
    /// The public HTTPS URL from Cloudflare.
    pub public_url: String,
    server_task: JoinHandle<()>,
    _child: Child,
}

impl Drop for TunnelGuard {
    fn drop(&mut self) {
        self.server_task.abort();
    }
}

/// Spawns a local sshx server and exposes it via a Cloudflare quick tunnel.
pub async fn start_cloudflare_tunnel() -> Result<TunnelGuard> {
    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .context("failed to bind ephemeral port for local server")?;
    let local_addr = listener.local_addr()?;
    let local_endpoint = format!("http://127.0.0.1:{}", local_addr.port());

    info!("Spawning cloudflared tunnel...");
    let mut child = Command::new("cloudflared")
        .arg("tunnel")
        .arg("--url")
        .arg(&local_endpoint)
        .stderr(Stdio::piped())
        .kill_on_drop(true)
        .spawn()
        .context("failed to execute `cloudflared`; make sure it is installed and in your PATH")?;

    let stderr = child.stderr.take().unwrap();
    let mut reader = BufReader::new(stderr).lines();

    let (url_tx, mut url_rx) = mpsc::channel(1);

    // Drain cloudflared stderr in background so it never gets a broken pipe.
    // We send the URL once we find it but keep reading to keep the pipe open.
    tokio::spawn(async move {
        let mut found = false;
        while let Ok(Some(line)) = reader.next_line().await {
            tracing::debug!("[cloudflared] {}", line);
            if !found {
                if let Some(idx) = line.find("https://") {
                    let sub = &line[idx..];
                    let end_idx = sub
                        .find(|c: char| c.is_whitespace() || c == '|' || c == ']')
                        .unwrap_or(sub.len());
                    let url = &sub[..end_idx];
                    if url.ends_with(".trycloudflare.com") || url.ends_with(".cloudflare.com") {
                        let _ = url_tx.send(url.to_string()).await;
                        found = true;
                        // keep reading — don't break, so cloudflared's pipe stays open
                    }
                }
            }
        }
    });

    let public_url = match timeout(Duration::from_secs(15), url_rx.recv()).await {
        Ok(Some(url)) => url,
        Ok(None) => return Err(anyhow!("cloudflared closed stderr without printing a tunnel URL")),
        Err(_) => return Err(anyhow!("timeout waiting for cloudflared public URL")),
    };

    info!("Tunnel public URL: {}", public_url);

    let mut options = ServerOptions::default();
    options.override_origin = Some(public_url.clone());

    let (tx, rx) = oneshot::channel();
    let local_endpoint_clone = local_endpoint.clone();
    let server_task = tokio::spawn(async move {
        let server = match Server::new(options) {
            Ok(s) => s,
            Err(e) => {
                let _ = tx.send(Err(e));
                return;
            }
        };
        let _ = tx.send(Ok(()));
        
        info!("Local sshx server listening on {}", local_endpoint_clone);
        if let Err(err) = server.listen(listener).await {
            warn!("Local server exited with error: {:?}", err);
        }
    });

    rx.await.context("local server failed to start")??;

    Ok(TunnelGuard {
        local_endpoint,
        public_url,
        server_task,
        _child: child,
    })
}
