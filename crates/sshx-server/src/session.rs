//! Core logic for sshx sessions, independent of message transport.

use std::sync::Arc;

use dashmap::DashMap;

/// In-memory state for a single sshx session.
#[derive(Default)]
pub struct Session {}

impl Session {
    /// Construct a new session.
    pub fn new() -> Self {
        Default::default()
    }
}

/// A concurrent map of session IDs to session objects.
pub type SessionStore = Arc<DashMap<String, Session>>;
