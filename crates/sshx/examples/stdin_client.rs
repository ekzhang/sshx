use std::io::Read;
use std::sync::Arc;
use std::thread;

use anyhow::Result;
use sshx::terminal::{get_default_shell, Terminal};
use tokio::io::{self, AsyncReadExt, AsyncWriteExt};
use tokio::signal;
use tokio::sync::mpsc;
use tracing::{error, info, trace};

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let shell = get_default_shell().await;
    info!(%shell, "using default shell");

    let mut terminal = Terminal::new(&shell).await?;

    // Separate thread for reading from standard input.
    let (tx, mut rx) = mpsc::channel::<Arc<[u8]>>(16);
    thread::spawn(move || loop {
        let mut buf = [0u8; 256];
        let n = std::io::stdin().read(&mut buf).unwrap();
        if tx.blocking_send(buf[0..n].into()).is_err() {
            break;
        }
    });

    let exit_signal = signal::ctrl_c();
    tokio::pin!(exit_signal);

    loop {
        let mut buf = [0u8; 256];

        tokio::select! {
            Some(bytes) = rx.recv() => {
                terminal.write_all(&bytes).await?;
            }
            result = terminal.read(&mut buf) => {
                let n = result?;
                io::stdout().write_all(&buf[..n]).await?;
            }
            result = &mut exit_signal => {
                if let Err(err) = result {
                    error!(?err, "failed to listen for exit signal");
                }
                trace!("gracefully exiting main");
                break;
            }
        }
    }

    Ok(())
}
