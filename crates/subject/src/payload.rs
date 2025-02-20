use serde::{Deserialize, Serialize};

use crate::subject::IntoSubject;

#[derive(Debug, Clone, thiserror::Error, PartialEq, Eq)]
pub enum SubjectPayloadError {
    #[error("Failed to encode or decode payload: {0}")]
    Serialization(String),
    #[error("Failed to deserialize payload: {0}")]
    Deserialization(String),
    #[error("Invalid subject parameters: {0}")]
    InvalidParams(String),
    #[error("Expected JSON object")]
    ExpectedJsonObject,
}

#[derive(Debug, Clone, Serialize, Deserialize, Hash, PartialEq, Eq)]
pub struct SubjectPayload {
    pub subject: String,
    pub params: serde_json::Value,
}

impl std::fmt::Display for SubjectPayload {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.subject, self.params)
    }
}

impl<T: IntoSubject> From<T> for SubjectPayload {
    fn from(value: T) -> Self {
        value.to_payload()
    }
}
