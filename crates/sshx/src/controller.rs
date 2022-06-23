//! Network gRPC client allowing server control of terminals.

use std::collections::HashMap;

use anyhow::{Context, Result};
use encoding_rs::{CoderResult, UTF_8};
use sshx_core::proto::{
    client_update::ClientMessage, server_update::ServerMessage,
    sshx_service_client::SshxServiceClient, ClientUpdate, CloseRequest, OpenRequest, TerminalData,
};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::sync::mpsc;
use tokio::time::{self, Duration, Instant, MissedTickBehavior};
use tokio_stream::{wrappers::ReceiverStream, StreamExt};
use tonic::transport::Channel;
use tracing::{error, info, warn};

use crate::terminal::Terminal;

/// Interval for sending empty heartbeat messages to the server.
const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(2);

/// Handles a singel session's communication with the remote server.
pub struct Controller {
    client: SshxServiceClient<Channel>,
    shell: String,

    name: String,
    token: String,
    url: String,

    /// Channels with backpressure routing messages to each shell task.
    shells_tx: HashMap<u32, mpsc::Sender<ShellData>>,
    /// Channel shared with tasks to allow them to output client messages.
    output_tx: mpsc::Sender<ClientMessage>,
    /// Owned receiving end of the `output_tx` channel.
    output_rx: mpsc::Receiver<ClientMessage>,
}

/// Internal message routed to shell tasks.
enum ShellData {
    /// Sequence of commands from the server.
    Data(String),
    /// Information about the server's current sequence number.
    Sync(u64),
}

impl Controller {
    /// Construct a new controller, connecting to the remote server.
    pub async fn new(origin: &str, shell: &str) -> Result<Self> {
        info!(%origin, "connecting to server");
        let mut client = SshxServiceClient::connect(String::from(origin)).await?;
        let req = OpenRequest {
            origin: origin.into(),
        };
        let resp = client.open(req).await?.into_inner();
        let (output_tx, output_rx) = mpsc::channel(64);
        Ok(Self {
            client,
            shell: shell.into(),
            name: resp.name,
            token: resp.token,
            url: resp.url,
            shells_tx: HashMap::new(),
            output_tx,
            output_rx,
        })
    }

    /// Returns the name of the session.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the URL of the session.
    pub fn url(&self) -> &str {
        &self.url
    }

    /// Run the controller forever, listening for requests from the server.
    pub async fn run(&mut self) -> ! {
        let mut last_retry = Instant::now();
        let mut retries = 0;
        loop {
            if let Err(err) = self.try_channel().await {
                if last_retry.elapsed() >= Duration::from_secs(10) {
                    retries = 0;
                }
                let secs = 2_u64.pow(retries.min(5));
                error!(?err, "disconnected, retrying in {secs}s...");
                time::sleep(Duration::from_secs(secs)).await;
                retries += 1;
            }
            last_retry = Instant::now();
        }
    }

    /// Helper function used by `run()` that can return errors.
    async fn try_channel(&mut self) -> Result<()> {
        let (tx, rx) = mpsc::channel(16);

        let hello = ClientMessage::Hello(format!("{},{}", self.name, self.token));
        send_msg(&tx, hello).await?;

        let resp = self.client.channel(ReceiverStream::new(rx)).await?;
        let mut messages = resp.into_inner(); // A stream of server messages.

        let mut interval = time::interval(HEARTBEAT_INTERVAL);
        interval.set_missed_tick_behavior(MissedTickBehavior::Delay);
        loop {
            let message = tokio::select! {
                _ = interval.tick() => {
                    tx.send(ClientUpdate::default()).await?;
                    continue;
                }
                msg = self.output_rx.recv() => {
                    let msg = msg.context("unreachable: output_tx was closed?")?;
                    send_msg(&tx, msg).await?;
                    continue;
                }
                item = messages.next() => {
                    item.context("server closed connection")?
                        .context("error from server")?
                        .server_message
                        .context("server message is missing")?
                }
            };

            match message {
                ServerMessage::Data(data) => {
                    // We ignore `data.seq` because it should be unused here.
                    if let Some(sender) = self.shells_tx.get(&data.id) {
                        // This line applies backpressure if the shell task is overloaded.
                        sender.send(ShellData::Data(data.data)).await.ok();
                    } else {
                        warn!(%data.id, "received data for non-existing shell");
                    }
                }
                ServerMessage::CreateShell(id) => {
                    if !self.shells_tx.contains_key(&id) {
                        self.spawn_shell_task(id);
                    } else {
                        warn!(%id, "server asked to create duplicate shell");
                    }
                }
                ServerMessage::CloseShell(id) => {
                    // Closes the channel when it is dropped, notifying the task to shut down.
                    self.shells_tx.remove(&id);
                    send_msg(&tx, ClientMessage::ClosedShell(id)).await?;
                }
                ServerMessage::Sync(seqnums) => {
                    for (id, seq) in seqnums.map {
                        if let Some(sender) = self.shells_tx.get(&id) {
                            sender.send(ShellData::Sync(seq)).await.ok();
                        } else {
                            warn!(%id, "received sequence number for non-existing shell");
                            send_msg(&tx, ClientMessage::ClosedShell(id)).await?;
                        }
                    }
                }
                ServerMessage::Error(err) => {
                    error!(?err, "error received from server");
                }
            }
        }
    }

    /// Entry point to start a new terminal task on the client.
    fn spawn_shell_task(&mut self, id: u32) {
        let (shell_tx, shell_rx) = mpsc::channel(16);
        let opt = self.shells_tx.insert(id, shell_tx);
        debug_assert!(opt.is_none(), "shell ID cannot be in existing tasks");

        let shell = self.shell.clone();
        let output_tx = self.output_tx.clone();
        tokio::spawn(async move {
            if let Err(err) = shell_task(id, &shell, shell_rx, output_tx.clone()).await {
                let err = ClientMessage::Error(err.to_string());
                output_tx.send(err).await.ok();
            }
            output_tx.send(ClientMessage::ClosedShell(id)).await.ok();
        });
    }

    /// Terminate this session gracefully.
    pub async fn close(&self) -> Result<bool> {
        info!("closing session");
        let req = CloseRequest {
            name: self.name.clone(),
            token: self.token.clone(),
        };
        let resp = self.client.clone().close(req).await?.into_inner();
        Ok(resp.exists)
    }
}

/// Attempt to send a client message to the server.
async fn send_msg(tx: &mpsc::Sender<ClientUpdate>, message: ClientMessage) -> Result<()> {
    let update = ClientUpdate {
        client_message: Some(message),
    };
    tx.send(update)
        .await
        .context("failed to send message to server")
}

/// Asynchronous task handling a single shell within the session.
async fn shell_task(
    id: u32,
    shell: &str,
    mut shell_rx: mpsc::Receiver<ShellData>,
    output_tx: mpsc::Sender<ClientMessage>,
) -> Result<()> {
    info!(%shell, "spawning new shell");
    output_tx.send(ClientMessage::CreatedShell(id)).await?;

    let mut term = Terminal::new(shell).await?;
    term.set_winsize(24, 80)?; // TODO: Make this reactive.

    let mut content = String::new(); // content from the terminal
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
                        term.write_all(data.as_bytes()).await?;
                    }
                    Some(ShellData::Sync(seq2)) => {
                        if seq2 < seq as u64 {
                            seq_outdated += 1;
                            if seq_outdated >= 3 {
                                seq = seq2 as usize;
                            }
                        }
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
        if content.len() > seq {
            seq = prev_char_boundary(&content, seq);
            let data = TerminalData {
                id,
                data: content[seq..].into(),
                seq: seq as u64,
            };
            output_tx.send(ClientMessage::Data(data)).await?;
            seq = content.len();
            seq_outdated = 0;
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
