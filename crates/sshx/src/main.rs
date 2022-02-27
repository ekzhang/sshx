use std::env;
use std::io::Read;
use std::os::unix::io::{FromRawFd, RawFd};
use std::pin::Pin;
use std::process::{Child, Command};
use std::sync::Arc;
use std::task::{Context, Poll};

use anyhow::Result;
use nix::pty;
use pin_project::{pin_project, pinned_drop};
use tokio::fs::File;
use tokio::io::{self, AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};
use tokio::sync::mpsc;

/// Returns the default shell on this system.
fn get_default_shell() -> String {
    env::var("SHELL").unwrap_or_else(|_| String::from("/bin/bash"))
}

/// An object that stores the state for a terminal session.
#[pin_project(PinnedDrop)]
pub struct Terminal {
    child: Child,
    #[pin]
    master_read: File,
    #[pin]
    master_write: File,
}

impl Terminal {
    /// Create a new terminal, with attached PTY.
    pub async fn new(shell: &str) -> Result<Terminal> {
        let result = pty::openpty(None, None)?;

        // Safety: The slave file descriptor was created by openpty() and has its
        // ownership transferred here. It is closed at the end of the function.
        let child = unsafe { Self::child_task(shell, result.slave) }?;

        // Safety: The master file descriptor was created by openpty() and has its
        // ownership transferred here. It is closed when the object is dropped.
        let master_read = unsafe { File::from_raw_fd(result.master) };

        // We need to clone the file object to prevent livelocks in Tokio, when multiple
        // reads and writes happen concurrently on the same file descriptor. This is a
        // current limitation of how the `tokio::fs::File` struct is implemented, due to
        // its blocking I/O on a separate thread.
        let master_write = master_read.try_clone().await?;

        Ok(Self {
            child,
            master_read,
            master_write,
        })
    }

    /// Entry point for the child process, which spawns a shell.
    unsafe fn child_task(shell: &str, slave_port: RawFd) -> Result<Child> {
        let slave = std::fs::File::from_raw_fd(slave_port);

        Command::new(shell)
            .stdin(slave.try_clone()?)
            .stdout(slave.try_clone()?)
            .stderr(slave)
            .spawn()
            .map_err(|e| e.into())
    }
}

// Redirect terminal reads to the read file object.
impl AsyncRead for Terminal {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut io::ReadBuf<'_>,
    ) -> Poll<io::Result<()>> {
        self.project().master_read.poll_read(cx, buf)
    }
}

// Redirect terminal reads to the write file object.
impl AsyncWrite for Terminal {
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<io::Result<usize>> {
        self.project().master_write.poll_write(cx, buf)
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        self.project().master_write.poll_flush(cx)
    }

    fn poll_shutdown(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        self.project().master_write.poll_shutdown(cx)
    }
}

#[pinned_drop]
impl PinnedDrop for Terminal {
    fn drop(self: Pin<&mut Self>) {
        // Reap the child process on closure so that it doesn't create zombies.
        self.project().child.kill().ok();
    }
}

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
