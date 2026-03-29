use uuid::Uuid;

#[derive(Debug, thiserror::Error)]
pub enum OutboxError {
    #[error("invalid event type: must be non-empty")]
    InvalidEventType,

    #[error("configuration error: {0}")]
    Config(String),

    #[error("database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("broker connection error: {0}")]
    BrokerConnection(#[from] lapin::Error),

    #[error("serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("broker nacked message: {0}")]
    BrokerNack(Uuid),

    #[error("publisher confirms not enabled on channel")]
    ConfirmsNotEnabled,
}
