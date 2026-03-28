
#[derive(Debug, thiserror::Error)]
pub enum OutboxError {
    #[error("invalid event type: must be non-empty")]
    InvalidEventType,
    #[error("configuration error: {0}")]
    Config(String),
}