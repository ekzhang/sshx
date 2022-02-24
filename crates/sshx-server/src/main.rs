use anyhow::Result;
use sshx_server::make_server_bind;
use tokio::signal::unix::{signal, SignalKind};

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let addr = "[::1]:8051".parse()?;

    let mut sigterm = signal(SignalKind::terminate())?;
    let mut sigint = signal(SignalKind::interrupt())?;

    tracing::info!("server listening at {addr}");
    make_server_bind(&addr, async {
        tokio::select! {
            _ = sigterm.recv() => (),
            _ = sigint.recv() => (),
        }
        tracing::info!("gracefully shutting down...");
    })
    .await?;
    Ok(())
}
