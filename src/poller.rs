use crate::{
    db::Db,
    errors::OutboxError,
    types::{EventType, MessageStatus, OutboxMessage},
};
use chrono::DateTime;
use tokio::time::Duration;

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
}
