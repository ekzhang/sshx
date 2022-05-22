//! Core global server state object, independent of transport.

use std::sync::Arc;

use dashmap::DashMap;
use hmac::{Hmac, Mac};
use sha2::Sha256;

use crate::session::Session;

/// Shared state object for global server logic.
pub struct ServerState {
    /// Message authentication code for signing tokens.
    pub mac: Hmac<Sha256>,

    /// A concurrent map of session IDs to session objects.
    pub store: DashMap<String, Arc<Session>>,
}

impl ServerState {
    /// Create an empty server state using the given secret.
    pub fn new(secret: &str) -> Self {
        let mac = Hmac::new_from_slice(secret.as_bytes()).expect("HMAC can take key of any size");
        Self {
            mac,
            store: DashMap::new(),
        }
    }
}
