use chrono::{DateTime, Utc};
use serde_json::Value;
use uuid::Uuid;

use crate::errors::OutboxError;


#[derive(Debug)]
pub enum MessageStatus {
    Pending,
    Published,
    Failed
}

#[derive(Debug)]
pub struct EventType(String);


#[derive(Debug)]
pub struct OutboxMessage {
    pub id: Uuid,
    pub event_type: EventType,
    pub payload: Value,
    pub aggregate_id: String,
    pub status: MessageStatus,
    pub attempts: i32,
    pub published_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>
}


impl EventType {
    pub fn new(s: impl Into<String>) -> Result<Self, OutboxError>{
        let s = s.into();
        if s.trim().is_empty() {
            return Err(OutboxError::InvalidEventType)
        }
        Ok(Self(s))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl OutboxMessage {
    pub fn new(event_type: EventType, payload: Value, aggregate_id: impl Into<String>) -> Self {
        Self {
            id: Uuid::new_v4(),
            event_type,
            payload,
            aggregate_id: aggregate_id.into(),
            status: MessageStatus::Pending,
            attempts: 0,
            published_at: None,
            created_at: Utc::now()
        }
    }
}

impl TryFrom<&str> for MessageStatus {
    type Error = OutboxError;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        match s {
            "pending" => Ok(Self::Pending),
            "failed" => Ok(Self::Failed),
            "published" => Ok(Self::Published),
            _=> return Err(OutboxError::Config(format!("unknown status: {s}")))
        }
    }

}

impl std::fmt::Display for MessageStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MessageStatus::Pending    => write!(f, "pending"),
            MessageStatus::Published  => write!(f, "published"),
            MessageStatus::Failed     => write!(f, "failed"),
        }
    }
}

impl std::fmt::Display for EventType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}