//! Utility functions shared among server logic.

use std::fmt::Debug;
use std::future::Future;
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
    pub fn wait(&'_ self) -> impl Future<Output = ()> + Send {
        let inner = self.inner.clone();
        async move {
            // Initial fast check
            if !inner.0.load(Ordering::Relaxed) {
                let notify = inner.1.notified();
                // Second check to avoid "missed wakeup" race conditions
                if !inner.0.load(Ordering::Relaxed) {
                    notify.await;
                }
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
