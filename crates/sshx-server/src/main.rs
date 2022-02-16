use sshx_server::make_server;
use tokio::signal::unix::{signal, SignalKind};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let addr = "[::1]:8051".parse()?;

    let mut sigterm = signal(SignalKind::terminate())?;
    let mut sigint = signal(SignalKind::interrupt())?;

    make_server(&addr, async {
        tokio::select! {
            _ = sigterm.recv() => (),
            _ = sigint.recv() => (),
        }
    })
    .await?;
    Ok(())
}
