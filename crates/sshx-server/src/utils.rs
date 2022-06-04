//! Utility functions shared among server logic.

use std::fmt::Debug;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use tokio::sync::Notify;

/// A cloneable structure that handles shutdown signals.
#[derive(Clone)]
pub struct Shutdown {
    inner: Arc<(AtomicBool, Notify)>,
}

impl Shutdown {
    /// Construct a new [`Shutdown`] object.
    pub fn new() -> Self {
        Self {
            inner: Arc::new((AtomicBool::new(false), Notify::new())),
        }
    }

    /// Send a shutdown signal to all listeners.
    pub fn shutdown(&self) {
        self.inner.0.swap(true, Ordering::Relaxed);
        self.inner.1.notify_waiters();
    }

    /// Returns whether the shutdown signal has been previously sent.
    pub fn is_terminated(&self) -> bool {
        self.inner.0.load(Ordering::Relaxed)
    }

    /// Wait for the shutdown signal, if it has not already been sent.
    pub async fn wait(&self) {
        // Initial fast check
        if !self.is_terminated() {
            let notify = self.inner.1.notified();
            // Second check to avoid "missed wakeup" race conditions
            if !self.is_terminated() {
                notify.await;
            }
        }
    }
}

impl Default for Shutdown {
    fn default() -> Self {
        Self::new()
    }
}

impl Debug for Shutdown {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Shutdown")
            .field("is_terminated", &self.inner.0.load(Ordering::Relaxed))
            .finish()
    }
}
