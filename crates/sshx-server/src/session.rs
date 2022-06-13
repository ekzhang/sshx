//! Core logic for sshx sessions, independent of message transport.

use std::collections::HashMap;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;

use anyhow::{bail, Context, Result};
use dashmap::DashMap;
use parking_lot::Mutex;
use sshx_core::proto::server_update::ServerMessage;
use tokio::sync::{watch, Notify};
use tokio::time::Instant;
use tokio_stream::{wrappers::WatchStream, Stream};
use tracing::info;

use crate::utils::Shutdown;

/// In-memory state for a single sshx session.
#[derive(Debug)]
pub struct Session {
    /// In-memory state for the session.
    shells: DashMap<u32, State>,

    /// Atomic counter to get new, unique shell IDs.
    counter: AtomicU32,

    /// Read-only timestamp when the session was started.
    created: Instant,

    /// Timestamp of the last client message from an active connection.
    updated: Mutex<Instant>,

    /// Watch channel source for the sorted list of open shells.
    ids: watch::Sender<Vec<u32>>,

    /// Sender end of a channel that buffers messages for the client.
    update_tx: async_channel::Sender<ServerMessage>,

    /// Receiver end of a channel that buffers messages for the client.
    update_rx: async_channel::Receiver<ServerMessage>,

    /// Set when this session has been closed and removed.
    shutdown: Shutdown,
}

/// Internal state for each shell.
#[derive(Default, Debug)]
struct State {
    /// Sequence number, indicating how many bytes have been received.
    seqnum: u64,

    /// Terminal data chunks, with associated timestamps in milliseconds.
    data: Vec<(u64, String)>,

    /// Set when this shell is terminated.
    closed: bool,

    /// Updated when any of the above fields change.
    notify: Arc<Notify>,
}

impl Session {
    /// Construct a new session.
    pub fn new() -> Self {
        let now = Instant::now();
        let (update_tx, update_rx) = async_channel::bounded(256);
        Session {
            shells: Default::default(),
            counter: AtomicU32::new(0),
            created: now,
            updated: Mutex::new(now),
            ids: watch::channel(Vec::new()).0,
            update_tx,
            update_rx,
            shutdown: Shutdown::new(),
        }
    }

    /// Returns the next shell ID.
    pub fn next_id(&self) -> u32 {
        self.counter.fetch_add(1, Ordering::Relaxed)
    }

    /// Return the sequence numbers for current shells.
    pub fn sequence_numbers(&self) -> HashMap<u32, u64> {
        let mut seqnums = HashMap::with_capacity(self.shells.len());
        for entry in &self.shells {
            if !entry.value().closed {
                seqnums.insert(*entry.key(), entry.value().seqnum);
            }
        }
        seqnums
    }

    /// Receive a notification every time the set of shells is changed.
    pub fn subscribe_shells(&self) -> impl Stream<Item = Vec<u32>> + 'static {
        WatchStream::new(self.ids.subscribe())
    }

    /// Subscribe for chunks from a shell, until it is closed.
    pub fn subscribe_chunks(
        &self,
        id: u32,
        chunknum: u64,
    ) -> impl Stream<Item = Vec<(u64, String)>> + '_ {
        let mut chunknum = chunknum as usize;
        async_stream::stream! {
            while !self.shutdown.is_terminated() {
                // We absolutely cannot hold `shell` across an await point,
                // since that would cause deadlocks.
                let shell = match self.shells.get(&id) {
                    Some(shell) if !shell.closed => shell,
                    _ => return,
                };
                let notify = Arc::clone(&shell.notify);
                let notified = notify.notified();
                if chunknum < shell.data.len() {
                    let chunks = shell.data[chunknum..].to_vec();
                    chunknum = shell.data.len();
                    drop(shell);
                    yield chunks;
                } else {
                    drop(shell);
                }

                tokio::select! {
                    _ = notified => (),
                    _ = self.terminated() => return,
                }
            }
        }
    }

    /// Add a new shell to the session.
    pub fn add_shell(&self, id: u32) -> Result<()> {
        use dashmap::mapref::entry::Entry::*;
        match self.shells.entry(id) {
            Occupied(_) => bail!("shell already exists with id={id}"),
            Vacant(v) => v.insert(State::default()),
        };
        self.ids.send_modify(|ids| {
            let index = ids.partition_point(|&x| x < id);
            ids.insert(index, id);
        });
        Ok(())
    }

    /// Terminates an existing shell.
    pub fn close_shell(&self, id: u32) -> Result<()> {
        match self.shells.get_mut(&id) {
            Some(mut shell) if !shell.closed => {
                shell.closed = true;
                shell.notify.notify_waiters();
            }
            Some(_) => return Ok(()),
            None => bail!("cannot close shell with id={id}, does not exist"),
        }
        self.ids.send_modify(|ids| {
            ids.retain(|&x| x != id);
        });
        Ok(())
    }

    /// Receive new data into the session.
    pub fn add_data(&self, id: u32, data: &str, seq: u64) -> Result<()> {
        let mut shell = match self.shells.get_mut(&id) {
            Some(shell) if !shell.closed => shell,
            Some(_) => bail!("cannot add data to shell with id={id}, already closed"),
            None => bail!("cannot add data to shell with id={id}, does not exist"),
        };

        debug_assert!(!shell.closed); // guaranteed by line above
        if seq <= shell.seqnum && seq + data.len() as u64 > shell.seqnum {
            let start = shell.seqnum - seq;
            let segment = data
                .get(start as usize..)
                .context("failed to decode utf-8 suffix in data")?;
            info!(id, ?segment, "adding data to shell");
            shell.data.push((
                self.created.elapsed().as_millis() as u64,
                String::from(segment),
            ));
            shell.seqnum += segment.len() as u64;
            shell.notify.notify_waiters();
        }

        Ok(())
    }

    /// Register a client message, refreshing the last update timestamp.
    pub fn access(&self) {
        *self.updated.lock() = Instant::now();
    }

    /// Access the sender of the client message channel for this session.
    pub fn update_tx(&self) -> &async_channel::Sender<ServerMessage> {
        &self.update_tx
    }

    /// Access the receiver of the client message channel for this session.
    pub fn update_rx(&self) -> &async_channel::Receiver<ServerMessage> {
        &self.update_rx
    }

    /// Send a termination signal to exit this session.
    pub fn shutdown(&self) {
        self.shutdown.shutdown()
    }

    /// Resolves when the session has received a shutdown signal.
    pub async fn terminated(&self) {
        self.shutdown.wait().await
    }
}

impl Default for Session {
    fn default() -> Self {
        Self::new()
    }
}
