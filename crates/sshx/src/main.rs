use std::io::Read;
use std::sync::Arc;

use anyhow::Result;
use sshx::{get_default_shell, Terminal};
use tokio::io::{self, AsyncReadExt, AsyncWriteExt};
use tokio::sync::mpsc;

#[tokio::main]
async fn main() -> Result<()> {
    let shell = get_default_shell();
    println!("Using default shell: {shell}");

    let mut terminal = Terminal::new(&shell).await?;

    // Separate thread for reading from standard input.
    let (tx, mut rx) = mpsc::channel::<Arc<[u8]>>(16);
    tokio::task::spawn_blocking(move || loop {
        let mut buf = [0_u8; 256];
        let n = std::io::stdin().read(&mut buf).unwrap();
        tx.blocking_send(buf[0..n].into()).unwrap();
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
        }
    }
}
