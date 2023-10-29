//! Defines tasks that control the behavior of a single shell in the client.

use anyhow::Result;
use encoding_rs::{CoderResult, UTF_8};
use sshx_core::proto::{client_update::ClientMessage, TerminalData};
use sshx_core::Sid;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    sync::mpsc,
};

use crate::encrypt::Encrypt;
use crate::terminal::Terminal;

const CONTENT_CHUNK_SIZE: usize = 1 << 16; // Send at most this many bytes at a time.
const CONTENT_ROLLING_BYTES: usize = 8 << 20; // Store at least this much content.
const CONTENT_PRUNE_BYTES: usize = 12 << 20; // Prune when we exceed this length.

/// Variants of terminal behavior that are used by the controller.
#[derive(Debug, Clone)]
pub enum Runner {
    /// Spawns the specified shell as a subprocess, forwarding PTYs.
    Shell(String),

    /// Mock runner that only echos its input, useful for testing.
    Echo,
}

/// Internal message routed to shell runners.
pub enum ShellData {
    /// Sequence of input bytes from the server.
    Data(Vec<u8>),
    /// Information about the server's current sequence number.
    Sync(u64),
    /// Resize the shell to a different number of rows and columns.
    Size(u32, u32),
}

impl Runner {
    /// Asynchronous task to run a single shell with process I/O.
    pub async fn run(
        &self,
        id: Sid,
        encrypt: Encrypt,
        shell_rx: mpsc::Receiver<ShellData>,
        output_tx: mpsc::Sender<ClientMessage>,
    ) -> Result<()> {
        match self {
            Self::Shell(shell) => shell_task(id, encrypt, shell, shell_rx, output_tx).await,
            Self::Echo => echo_task(id, encrypt, shell_rx, output_tx).await,
        }
    }
}

/// Asynchronous task handling a single shell within the session.
async fn shell_task(
    id: Sid,
    encrypt: Encrypt,
    shell: &str,
    mut shell_rx: mpsc::Receiver<ShellData>,
    output_tx: mpsc::Sender<ClientMessage>,
) -> Result<()> {
    let mut term = Terminal::new(shell).await?;
    term.set_winsize(24, 80)?;

    let mut content = String::new(); // content from the terminal
    let mut content_offset = 0; // bytes before the first character of `content`
    let mut decoder = UTF_8.new_decoder(); // UTF-8 streaming decoder
    let mut seq = 0; // our log of the server's sequence number
    let mut seq_outdated = 0; // number of times seq has been outdated
    let mut buf = [0u8; 4096]; // buffer for reading
    let mut finished = false; // set when this is done

    while !finished {
        tokio::select! {
            result = term.read(&mut buf) => {
                let n = result?;
                if n == 0 {
                    finished = true;
                } else {
                    content.reserve(decoder.max_utf8_buffer_length(n).unwrap());
                    let (result, _, _) = decoder.decode_to_string(&buf[..n], &mut content, false);
                    debug_assert!(result == CoderResult::InputEmpty);
                }
            }
            item = shell_rx.recv() => {
                match item {
                    Some(ShellData::Data(data)) => {
                        term.write_all(&data).await?;
                    }
                    Some(ShellData::Sync(seq2)) => {
                        if seq2 < seq as u64 {
                            seq_outdated += 1;
                            if seq_outdated >= 3 {
                                seq = seq2 as usize;
                            }
                        }
                    }
                    Some(ShellData::Size(rows, cols)) => {
                        term.set_winsize(rows as u16, cols as u16)?;
                    }
                    None => finished = true, // Server closed this shell.
                }
            }
        }

        if finished {
            content.reserve(decoder.max_utf8_buffer_length(0).unwrap());
            let (result, _, _) = decoder.decode_to_string(&[], &mut content, true);
            debug_assert!(result == CoderResult::InputEmpty);
        }

        // Send data if the server has fallen behind.
        if content_offset + content.len() > seq {
            let start = prev_char_boundary(&content, seq - content_offset);
            let end = prev_char_boundary(&content, (start + CONTENT_CHUNK_SIZE).min(content.len()));
            let data = encrypt.segment(
                0x100000000 | id.0 as u64, // stream number
                (content_offset + start) as u64,
                content[start..end].as_bytes(),
            );
            let data = TerminalData {
                id: id.0,
                data: data.into(),
                seq: (content_offset + start) as u64,
            };
            output_tx.send(ClientMessage::Data(data)).await?;
            seq = content_offset + end;
            seq_outdated = 0;
        }

        if content.len() > CONTENT_PRUNE_BYTES && seq - CONTENT_ROLLING_BYTES > content_offset {
            let pruned = (seq - CONTENT_ROLLING_BYTES) - content_offset;
            let pruned = prev_char_boundary(&content, pruned);
            content_offset += pruned;
            content.drain(..pruned);
        }
    }
    Ok(())
}

/// Find the last char boundary before an index in O(1) time.
fn prev_char_boundary(s: &str, i: usize) -> usize {
    (0..=i)
        .rev()
        .find(|&j| s.is_char_boundary(j))
        .expect("no previous char boundary")
}

async fn echo_task(
    id: Sid,
    encrypt: Encrypt,
    mut shell_rx: mpsc::Receiver<ShellData>,
    output_tx: mpsc::Sender<ClientMessage>,
) -> Result<()> {
    let mut seq = 0;
    while let Some(item) = shell_rx.recv().await {
        match item {
            ShellData::Data(data) => {
                let msg = String::from_utf8_lossy(&data);
                let term_data = TerminalData {
                    id: id.0,
                    data: encrypt
                        .segment(0x100000000 | id.0 as u64, seq, msg.as_bytes())
                        .into(),
                    seq,
                };
                output_tx.send(ClientMessage::Data(term_data)).await?;
                seq += msg.len() as u64;
            }
            ShellData::Sync(_) => (),
            ShellData::Size(_, _) => (),
        }
    }
    Ok(())
}
