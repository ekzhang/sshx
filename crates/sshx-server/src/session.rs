//! Core logic for sshx sessions, independent of message transport.

use std::{collections::HashMap, sync::Arc};

use anyhow::{bail, Context, Result};
use dashmap::DashMap;
use tokio::{sync::watch, time::Instant};
use tracing::info;

/// In-memory state for a single sshx session.
#[derive(Debug)]
pub struct Session {
    /// In-memory state for the session.
    shells: DashMap<u32, State>,

    /// Read-only timestamp when the session was started.
    created: Instant,

    /// Watch channel source for new sequence numbers.
    seqnums: watch::Sender<HashMap<u32, u64>>,
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
}

impl Session {
    /// Construct a new session.
    pub fn new() -> Self {
        Session {
            shells: Default::default(),
            created: Instant::now(),
            seqnums: watch::channel(HashMap::default()).0,
        }
    }

    /// Return the sequence numbers for current shells.
    pub fn sequence_numbers(&self) -> HashMap<u32, u64> {
        self.seqnums.borrow().clone()
    }

    /// Add a new shell to the session.
    pub fn add_shell(&self, id: u32) -> Result<()> {
        use dashmap::mapref::entry::Entry::*;
        match self.shells.entry(id) {
            Occupied(_) => bail!("shell already exists with id={id}"),
            Vacant(v) => v.insert(State::default()),
        };
        self.seqnums.send_modify(|seqnums| {
            seqnums.insert(id, 0);
        });
        Ok(())
    }

    /// Terminates an existing shell.
    pub fn close_shell(&self, id: u32) -> Result<()> {
        match self.shells.get_mut(&id) {
            Some(mut shell) if !shell.closed => shell.closed = true,
            Some(_) => return Ok(()),
            None => bail!("cannot close shell with id={id}, does not exist"),
        }
        self.seqnums.send_modify(|seqnums| {
            seqnums.remove(&id);
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
            info!(id, n = segment.len(), "adding data to shell");
            shell.data.push((
                self.created.elapsed().as_millis() as u64,
                String::from(segment),
            ));
            shell.seqnum += segment.len() as u64;
        }

        Ok(())
    }
}

impl Default for Session {
    fn default() -> Self {
        Self::new()
    }
}

/// A concurrent map of session IDs to session objects.
pub type SessionStore = Arc<DashMap<String, Arc<Session>>>;
