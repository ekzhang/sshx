use std::pin::Pin;
use std::process::Command;
use std::task::Context;
use std::task::Poll;

use anyhow::Result;
use pin_project::{pin_project, pinned_drop};
use tokio::fs::{self, File};
use tokio::io::{self, AsyncRead, AsyncWrite};
use tracing::instrument;

/// Returns the default shell on this system.
///
/// For Windows, this is implemented currently to just look for shells at a
/// couple locations. If it fails, it returns `cmd.exe`.
///
/// Note: I can't get `powershell.exe` to work with ConPTY, since it returns
/// error 8009001d. There's some magic environment variables that need to be set
/// for Powershell to launch. This is why I don't typically use Windows!
pub async fn get_default_shell() -> String {
    for shell in [
        "C:\\Program Files\\Git\\bin\\bash.exe",
        "C:\\Windows\\System32\\cmd.exe",
    ] {
        if fs::metadata(shell).await.is_ok() {
            return shell.to_string();
        }
    }
    String::from("cmd.exe")
}

/// An object that stores the state for a terminal session.
#[pin_project(PinnedDrop)]
pub struct Terminal {
    child: conpty::Process,
    #[pin]
    reader: File,
    #[pin]
    writer: File,
    winsize: (u16, u16),
}

impl Terminal {
    /// Create a new terminal, with attached PTY.
    #[instrument]
    pub async fn new(shell: &str) -> Result<Terminal> {
        let mut command = Command::new(shell);

        // Set terminal environment variables appropriately.
        command.env("TERM", "xterm-256color");
        command.env("COLORTERM", "truecolor");
        command.env("TERM_PROGRAM", "sshx");
        command.env_remove("TERM_PROGRAM_VERSION");

        let mut child =
            tokio::task::spawn_blocking(move || conpty::Process::spawn(command)).await??;
        let reader = File::from_std(child.output()?.into());
        let writer = File::from_std(child.input()?.into());

        Ok(Self {
            child,
            reader,
            writer,
            winsize: (0, 0),
        })
    }

    /// Get the window size of the TTY.
    pub fn get_winsize(&self) -> Result<(u16, u16)> {
        Ok(self.winsize)
    }

    /// Set the window size of the TTY.
    pub fn set_winsize(&mut self, rows: u16, cols: u16) -> Result<()> {
        let rows_i16 = rows.min(i16::MAX as u16) as i16;
        let cols_i16 = cols.min(i16::MAX as u16) as i16;
        self.child.resize(cols_i16, rows_i16)?; // Note argument order
        self.winsize = (rows, cols);
        Ok(())
    }
}

// Redirect terminal reads to the read file object.
impl AsyncRead for Terminal {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut io::ReadBuf<'_>,
    ) -> Poll<io::Result<()>> {
        self.project().reader.poll_read(cx, buf)
    }
}

// Redirect terminal writes to the write file object.
impl AsyncWrite for Terminal {
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<io::Result<usize>> {
        self.project().writer.poll_write(cx, buf)
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        self.project().writer.poll_flush(cx)
    }

    fn poll_shutdown(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        self.project().writer.poll_shutdown(cx)
    }
}

#[pinned_drop]
impl PinnedDrop for Terminal {
    fn drop(self: Pin<&mut Self>) {
        let this = self.project();
        this.child.exit(0).ok();
    }
}
