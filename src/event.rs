use crate::level::Level;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;

pub type Fields = serde_json::Map<String, Value>;

/// Information about an error captured in an event.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorInfo {
    pub name: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub why: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fix: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub link: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stack: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub internal: Option<Value>,
}

/// A structured log event.
///
/// Represents both simple log entries and wide events that accumulate context.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    pub level: Level,
    pub timestamp: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub service: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub environment: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub method: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration_ms: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tag: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<ErrorInfo>,
    #[serde(flatten)]
    pub fields: Fields,
}

impl Event {
    /// Create a new event with current timestamp and empty fields.
    pub fn new(level: Level) -> Self {
        Self {
            level,
            timestamp: Utc::now(),
            service: None,
            environment: None,
            method: None,
            path: None,
            request_id: None,
            duration_ms: None,
            status: None,
            tag: None,
            message: None,
            error: None,
            fields: serde_json::Map::new(),
        }
    }

    /// Create a simple tagged log event.
    pub fn simple(level: Level, tag: impl Into<String>, message: impl Into<String>) -> Self {
        let mut event = Self::new(level);
        event.tag = Some(tag.into());
        event.message = Some(message.into());
        event
    }

    /// Serialize this event to a JSON string.
    pub fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap_or_else(|_| "{}".to_string())
    }
}
