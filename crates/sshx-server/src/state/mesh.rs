//! Storage and distributed communication.

use std::{pin::pin, sync::Arc, time::Duration};

use anyhow::Result;
use redis::AsyncCommands;
use tokio::time;
use tokio_stream::{Stream, StreamExt};
use tracing::error;

use crate::session::Session;

/// Interval for syncing the latest session state into persistent storage.
const STORAGE_SYNC_INTERVAL: Duration = Duration::from_secs(20);

/// Length of time a key lasts in Redis before it is expired.
const STORAGE_EXPIRY: Duration = Duration::from_secs(300);

fn set_opts() -> redis::SetOptions {
    redis::SetOptions::default()
        .with_expiration(redis::SetExpiry::PX(STORAGE_EXPIRY.as_millis() as u64))
}

/// Communication with a distributed mesh of sshx server nodes.
///
/// This uses a Redis instance to persist data across restarts, as well as a
/// pub/sub channel to keep be notified of when another node becomes the owner
/// of an active session.
///
/// All servers must be accessible to each other through TCP mesh networking,
/// since requests are forwarded to the controller of a given session.
#[derive(Clone)]
pub struct StorageMesh {
    redis: deadpool_redis::Pool,
    redis_pubsub: redis::Client,
    host: Option<String>,
}

impl StorageMesh {
    /// Construct a new storage object from Redis URL.
    pub fn new(redis_url: &str, host: Option<&str>) -> Result<Self> {
        let redis = deadpool_redis::Config::from_url(redis_url)
            .builder()?
            .max_size(4)
            .wait_timeout(Some(Duration::from_secs(5)))
            .runtime(deadpool_redis::Runtime::Tokio1)
            .build()?;

        // Separate `redis::Client` just for pub/sub connections.
        //
        // At time of writing, deadpool-redis has not been updated to support the new
        // pub/sub client APIs in Rust. This is a temporary workaround that creates a
        // new Redis client on the side, bypassing the connection pool.
        //
        // Reference: https://github.com/deadpool-rs/deadpool/issues/226
        let redis_pubsub = redis::Client::open(redis_url)?;

        Ok(Self {
            redis,
            redis_pubsub,
            host: host.map(|s| s.to_string()),
        })
    }

    /// Returns the hostname of this server, if running in mesh node.
    pub fn host(&self) -> Option<&str> {
        self.host.as_deref()
    }

    /// Retrieve the hostname of the owner of a session.
    pub async fn get_owner(&self, name: &str) -> Result<Option<String>> {
        let mut conn = self.redis.get().await?;
        let (owner, closed) = redis::pipe()
            .get(format!("session:{{{name}}}:owner"))
            .get(format!("session:{{{name}}}:closed"))
            .query_async(&mut conn)
            .await?;
        if closed {
            Ok(None)
        } else {
            Ok(owner)
        }
    }

    /// Retrieve the owner and snapshot of a session.
    pub async fn get_owner_snapshot(
        &self,
        name: &str,
    ) -> Result<(Option<String>, Option<Vec<u8>>)> {
        let mut conn = self.redis.get().await?;
        let (owner, snapshot, closed) = redis::pipe()
            .get(format!("session:{{{name}}}:owner"))
            .get(format!("session:{{{name}}}:snapshot"))
            .get(format!("session:{{{name}}}:closed"))
            .query_async(&mut conn)
            .await?;
        if closed {
            Ok((None, None))
        } else {
            Ok((owner, snapshot))
        }
    }

    /// Periodically set the owner and snapshot of a session.
    pub async fn background_sync(&self, name: &str, session: Arc<Session>) {
        let mut interval = time::interval(STORAGE_SYNC_INTERVAL);
        interval.set_missed_tick_behavior(time::MissedTickBehavior::Delay);
        loop {
            tokio::select! {
                _ = interval.tick() => {}
                _ = session.sync_now_wait() => {}
                _ = session.terminated() => break,
            }
            let mut conn = match self.redis.get().await {
                Ok(conn) => conn,
                Err(err) => {
                    error!(?err, "failed to connect to redis for sync");
                    continue;
                }
            };
            let snapshot = match session.snapshot() {
                Ok(snapshot) => snapshot,
                Err(err) => {
                    error!(?err, "failed to snapshot session {name}");
                    continue;
                }
            };
            let mut pipe = redis::pipe();
            if let Some(host) = &self.host {
                pipe.set_options(format!("session:{{{name}}}:owner"), host, set_opts());
            }
            pipe.set_options(format!("session:{{{name}}}:snapshot"), snapshot, set_opts());
            match pipe.query_async(&mut conn).await {
                Ok(()) => {}
                Err(err) => error!(?err, "failed to sync session {name}"),
            }
        }
    }

    /// Mark a session as closed, so it will expire and never be accessed again.
    pub async fn mark_closed(&self, name: &str) -> Result<()> {
        let mut conn = self.redis.get().await?;
        let (owner,): (Option<String>,) = redis::pipe()
            .get_del(format!("session:{{{name}}}:owner"))
            .del(format!("session:{{{name}}}:snapshot"))
            .ignore()
            .set_options(format!("session:{{{name}}}:closed"), true, set_opts())
            .ignore()
            .query_async(&mut conn)
            .await?;
        if let Some(owner) = owner {
            self.notify_transfer(name, &owner).await?;
        }
        Ok(())
    }

    /// Notify a host that a session has been transferred.
    pub async fn notify_transfer(&self, name: &str, host: &str) -> Result<()> {
        let mut conn = self.redis.get().await?;
        () = conn.publish(format!("transfers:{host}"), name).await?;
        Ok(())
    }

    /// Listen for sessions that are transferred away from this host.
    pub fn listen_for_transfers(&self) -> impl Stream<Item = String> + Send + '_ {
        async_stream::stream! {
            let Some(host) = &self.host else {
                // If not in a mesh, there are no transfers.
                return;
            };

            loop {
                // Requires an owned, non-pool connection for ownership reasons.
                let mut pubsub = match self.redis_pubsub.get_async_pubsub().await {
                    Ok(pubsub) => pubsub,
                    Err(err) => {
                        error!(?err, "failed to connect to redis for pub/sub");
                        time::sleep(Duration::from_secs(5)).await;
                        continue;
                    }
                };
                if let Err(err) = pubsub.subscribe(format!("transfers:{host}")).await {
                    error!(?err, "failed to subscribe to transfers");
                    time::sleep(Duration::from_secs(1)).await;
                    continue;
                }

                let mut msg_stream = pin!(pubsub.into_on_message());
                while let Some(msg) = msg_stream.next().await {
                    match msg.get_payload::<String>() {
                        Ok(payload) => yield payload,
                        Err(err) => {
                            error!(?err, "failed to parse transfers message");
                            continue;
                        }
                    };
                }
            }
        }
    }
}
