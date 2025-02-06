use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, thiserror::Error, PartialEq, Eq)]
pub enum SubjectPayloadError {
    #[error("Failed to encode or decode payload: {0}")]
    Serialization(String),
    #[error("Failed to deserialize payload: {0}")]
    Deserialization(String),
}

#[derive(Debug, Clone, Serialize, Deserialize, Hash, PartialEq, Eq)]
pub struct SubjectPayload {
    pub subject: String,
    pub params: serde_json::Value,
}

impl std::fmt::Display for SubjectPayload {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.subject, self.params.to_string())
    }
}
