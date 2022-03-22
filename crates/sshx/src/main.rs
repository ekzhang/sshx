use std::io::Read;
use std::sync::Arc;
use std::thread;

use anyhow::Result;
use sshx::{get_default_shell, Terminal};
use tokio::io::{self, AsyncReadExt, AsyncWriteExt};
use tokio::signal;
use tokio::sync::{mpsc, watch};
use tracing::{info, trace};

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let shell = get_default_shell();
    info!(%shell, "using default shell");

    let mut terminal = Terminal::new(&shell).await?;

    // Separate thread for reading from standard input.
    let (tx, mut rx) = mpsc::channel::<Arc<[u8]>>(16);
    thread::spawn(move || loop {
        let mut buf = [0_u8; 256];
        let n = std::io::stdin().read(&mut buf).unwrap();
        if tx.blocking_send(buf[0..n].into()).is_err() {
            break;
        }
    });

    let (exit_tx, mut exit_rx) = watch::channel(false);
    tokio::spawn(async move {
        signal::ctrl_c().await.unwrap();
        exit_tx.send(true).ok();
    });

    loop {
        let mut buf = [0_u8; 256];

        tokio::select! {
            Some(bytes) = rx.recv() => {
                terminal.write_all(&bytes).await?;
            }
            result = terminal.read(&mut buf) => {
                let n = result?;
                io::stdout().write_all(&buf[..n]).await?;
            }
            Ok(()) = exit_rx.changed() => {
                if *exit_rx.borrow_and_update() {
                    trace!("gracefully exiting main");
                    break;
                }
            }
        }
    }

    Ok(())
}
