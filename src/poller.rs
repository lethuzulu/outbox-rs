use std::sync::Arc;

use crate::{
    db::Db,
    errors::OutboxError,
    publisher::Publisher,
};
use tokio::{
    sync::watch,
    time::{Duration, interval},
};

#[derive(Debug)]
pub struct PollerConfig {
    /// the number of messages to claim per poll cycle
    pub batch_size: i64,
    /// how often to poll for messages
    pub poll_interval: Duration,
    /// how long a row is invisible to other pollers
    /// if the poller crashes before publishing, the lock expires
    /// and another poller pucks up the row automatically
    pub lock_secs: i64,
}

#[derive(Debug)]
pub struct Poller {
    db: Db,
    config: PollerConfig,
}

impl Poller {
    pub fn new(db: Db, config: PollerConfig) -> Self {
        Self { db, config }
    }

    pub async fn run(
        &self,
        publisher: Arc<Publisher>,
        mut shutdown: watch::Receiver<bool>,
    ) -> Result<(), OutboxError> {
        let mut ticker = interval(self.config.poll_interval);

        loop {
            tokio::select! {
                _ = shutdown.changed() => {
                    if *shutdown.borrow() {return Ok(());}
                },
                _ = ticker.tick() => {
                    let messages = match self.db.poll(self.config.lock_secs, self.config.batch_size).await {
                        Ok(m) => m,
                        Err(e) => { tracing::error!(%e, "poll failed"); continue; }
                    };

                    for msg in messages {
                        match publisher.publish(&msg).await {
                            Ok(()) => {
                                // only mark published after broker confirms
                                self.db.mark_published(msg.id).await?;
                            },
                            Err(e) => {
                                tracing::warn!(id = %msg.id, error = %e, "publish failed");
                                self.db.mark_failed(msg.id, &e.to_string()).await?;
                            }
                        }
                    }
                }
            }
        }
    }
}
