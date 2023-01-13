//! Core logic for sshx sessions, independent of message transport.

use std::collections::HashMap;
use std::ops::DerefMut;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;

use anyhow::{bail, Context, Result};
use parking_lot::{Mutex, RwLock, RwLockWriteGuard};
use sshx_core::proto::server_update::ServerMessage;
use tokio::sync::{broadcast, watch, Notify};
use tokio::time::Instant;
use tokio_stream::wrappers::errors::BroadcastStreamRecvError;
use tokio_stream::wrappers::{BroadcastStream, WatchStream};
use tokio_stream::Stream;
use tracing::{debug, warn};

use crate::utils::Shutdown;
use crate::web::{WsServer, WsUser, WsWinsize};

/// In-memory state for a single sshx session.
#[derive(Debug)]
pub struct Session {
    /// In-memory state for the session.
    shells: RwLock<HashMap<u32, State>>,

    /// Metadata for currently connected users.
    users: RwLock<HashMap<u32, WsUser>>,

    /// Atomic counter to get new, unique IDs.
    counter: AtomicU32,

    /// Read-only timestamp when the session was started.
    created: Instant,

    /// Timestamp of the last client message from an active connection.
    updated: Mutex<Instant>,

    /// Watch channel source for the ordered list of open shells and sizes.
    source: watch::Sender<Vec<(u32, WsWinsize)>>,

    /// Broadcasts updates to all WebSocket clients.
    ///
    /// Every update inside this channel must be of idempotent form, since
    /// messages may arrive before or after any snapshot of the current session
    /// state. Duplicated events should remain consistent.
    broadcast: broadcast::Sender<WsServer>,

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
            shells: RwLock::new(HashMap::new()),
            users: RwLock::new(HashMap::new()),
            counter: AtomicU32::new(1),
            created: now,
            updated: Mutex::new(now),
            source: watch::channel(Vec::new()).0,
            broadcast: broadcast::channel(32).0,
            update_tx,
            update_rx,
            shutdown: Shutdown::new(),
        }
    }

    /// Returns the next unique ID (used for shells and users).
    pub fn next_id(&self) -> u32 {
        self.counter.fetch_add(1, Ordering::Relaxed)
    }

    /// Return the sequence numbers for current shells.
    pub fn sequence_numbers(&self) -> HashMap<u32, u64> {
        let shells = self.shells.read();
        let mut seqnums = HashMap::with_capacity(shells.len());
        for (key, value) in &*shells {
            if !value.closed {
                seqnums.insert(*key, value.seqnum);
            }
        }
        seqnums
    }

    /// Receive a notification on broadcasted message events.
    pub fn subscribe_broadcast(
        &self,
    ) -> impl Stream<Item = Result<WsServer, BroadcastStreamRecvError>> + Unpin {
        BroadcastStream::new(self.broadcast.subscribe())
    }

    /// Receive a notification every time the set of shells is changed.
    pub fn subscribe_shells(&self) -> impl Stream<Item = Vec<(u32, WsWinsize)>> + Unpin {
        WatchStream::new(self.source.subscribe())
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
                // We absolutely cannot hold `shells` across an await point,
                // since that would cause deadlocks.
                let (chunks, notified) = {
                    let shells = self.shells.read();
                    let shell = match shells.get(&id) {
                        Some(shell) if !shell.closed => shell,
                        _ => return,
                    };
                    let notify = Arc::clone(&shell.notify);
                    let notified = async move { notify.notified().await };
                    let mut chunks = Vec::new();
                    if chunknum < shell.data.len() {
                        chunks.extend_from_slice(&shell.data[chunknum..]);
                        chunknum = shell.data.len();
                    }
                    (chunks, notified)
                };

                if !chunks.is_empty() {
                    yield chunks;
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
        use std::collections::hash_map::Entry::*;
        let _guard = match self.shells.write().entry(id) {
            Occupied(_) => bail!("shell already exists with id={id}"),
            Vacant(v) => v.insert(State::default()),
        };
        self.source.send_modify(|source| {
            let winsize = match source.len() {
                0 => WsWinsize::default(),
                _ => WsWinsize::new_random(),
            };
            source.push((id, winsize));
        });
        Ok(())
    }

    /// Terminates an existing shell.
    pub fn close_shell(&self, id: u32) -> Result<()> {
        match self.shells.write().get_mut(&id) {
            Some(mut shell) if !shell.closed => {
                shell.closed = true;
                shell.notify.notify_waiters();
            }
            Some(_) => return Ok(()),
            None => bail!("cannot close shell with id={id}, does not exist"),
        }
        self.source.send_modify(|source| {
            source.retain(|&(x, _)| x != id);
        });
        Ok(())
    }

    fn get_shell_mut(&self, id: u32) -> Result<impl DerefMut<Target = State> + '_> {
        let shells = self.shells.write();
        match shells.get(&id) {
            Some(shell) if !shell.closed => {
                Ok(RwLockWriteGuard::map(shells, |s| s.get_mut(&id).unwrap()))
            }
            Some(_) => bail!("cannot update shell with id={id}, already closed"),
            None => bail!("cannot update shell with id={id}, does not exist"),
        }
    }

    /// Change the size of a terminal, notifying clients if necessary.
    pub fn move_shell(&self, id: u32, winsize: Option<WsWinsize>) -> Result<()> {
        let _guard = self.get_shell_mut(id)?; // Ensures mutual exclusion.
        self.source.send_modify(|source| {
            if let Some(idx) = source.iter().position(|&(sid, _)| sid == id) {
                let (_, oldsize) = source.remove(idx);
                source.push((id, winsize.unwrap_or(oldsize)));
            }
        });
        Ok(())
    }

    /// Receive new data into the session.
    pub fn add_data(&self, id: u32, data: &str, seq: u64) -> Result<()> {
        let mut shell = self.get_shell_mut(id)?;

        if seq <= shell.seqnum && seq + data.len() as u64 > shell.seqnum {
            let start = shell.seqnum - seq;
            let segment = data
                .get(start as usize..)
                .context("failed to decode utf-8 suffix in data")?;
            debug!(id, ?segment, "adding data to shell");
            shell.data.push((
                self.created.elapsed().as_millis() as u64,
                String::from(segment),
            ));
            shell.seqnum += segment.len() as u64;
            shell.notify.notify_waiters();
        }

        Ok(())
    }

    /// List all the users in the session.
    pub fn list_users(&self) -> Vec<(u32, WsUser)> {
        self.users
            .read()
            .iter()
            .map(|(k, v)| (*k, v.clone()))
            .collect()
    }

    /// Update a user in place by ID, applying a callback to the object.
    pub fn update_user(&self, id: u32, f: impl FnOnce(&mut WsUser)) -> Result<()> {
        let updated_user = {
            let mut users = self.users.write();
            let user = users.get_mut(&id).context("user not found")?;
            f(user);
            user.clone()
        };
        self.broadcast
            .send(WsServer::UserDiff(id, Some(updated_user)))
            .ok();
        Ok(())
    }

    /// Add a new user, and return a guard that removes the user when dropped.
    pub fn user_scope(&self, id: u32) -> Result<impl Drop + '_> {
        use std::collections::hash_map::Entry::*;

        #[must_use]
        struct UserGuard<'a>(&'a Session, u32);
        impl Drop for UserGuard<'_> {
            fn drop(&mut self) {
                self.0.remove_user(self.1);
            }
        }

        match self.users.write().entry(id) {
            Occupied(_) => bail!("user already exists with id={id}"),
            Vacant(v) => {
                let user = WsUser {
                    name: format!("User {id}"),
                    cursor: None,
                };
                v.insert(user.clone());
                self.broadcast.send(WsServer::UserDiff(id, Some(user))).ok();
                Ok(UserGuard(self, id))
            }
        }
    }

    /// Remove an existing user.
    fn remove_user(&self, id: u32) {
        if self.users.write().remove(&id).is_none() {
            warn!(%id, "invariant violation: removed user that does not exist");
        }
        self.broadcast.send(WsServer::UserDiff(id, None)).ok();
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
