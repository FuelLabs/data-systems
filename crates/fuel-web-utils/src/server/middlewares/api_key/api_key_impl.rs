use std::fmt;

use actix_web::{HttpMessage, HttpRequest};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::ApiKeyError;

#[derive(Debug, Serialize, Deserialize, Clone, Eq, PartialEq)]
pub enum ApiKeyLimit {
    Limited(u32),
    Unlimited,
}

impl fmt::Display for ApiKeyLimit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ApiKeyLimit::Limited(limit) => write!(f, "{}", limit),
            ApiKeyLimit::Unlimited => write!(f, "unlimited"),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Eq, PartialEq)]
pub struct ApiKeyLimits {
    pub max_reads_per_minute: ApiKeyLimit,
    pub max_writes_per_minute: ApiKeyLimit,
}

#[derive(Debug, Serialize, Deserialize, Clone, Eq, PartialEq)]
pub struct ApiKeyRestrictions {
    pub allowed_domains: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Eq, PartialEq)]
pub enum ApiKeyStatus {
    Active,
    Inactive,
    Deleted,
}

pub type ApiKeyId = u64;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct ApiKey {
    user_id: ApiKeyId,
    user_name: String,
    api_key: String,
    pub limits: ApiKeyLimits,
    pub restrictions: ApiKeyRestrictions,
    pub status: ApiKeyStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl ApiKey {
    pub fn new(user_id: ApiKeyId, user_name: String, api_key: String) -> Self {
        Self {
            user_id,
            user_name,
            api_key,
            limits: ApiKeyLimits {
                max_reads_per_minute: ApiKeyLimit::Limited(100),
                max_writes_per_minute: ApiKeyLimit::Limited(100),
            },
            restrictions: ApiKeyRestrictions {
                allowed_domains: vec![],
            },
            status: ApiKeyStatus::Active,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        }
    }

    pub fn from_req(req: &HttpRequest) -> Result<ApiKey, ApiKeyError> {
        match req.extensions().get::<ApiKey>() {
            Some(api_key) => {
                tracing::info!(
                    user_id = %api_key.user_id,
                    user_name = %api_key.user_name,
                    "Authenticated request"
                );
                Ok(api_key.to_owned())
            }
            None => {
                tracing::warn!("Unauthenticated request attempt");
                Err(ApiKeyError::NotFound)
            }
        }
    }

    pub fn validate_status(&self) -> Result<(), ApiKeyError> {
        match self.status {
            ApiKeyStatus::Active => Ok(()),
            ApiKeyStatus::Inactive => Err(ApiKeyError::Inactive),
            ApiKeyStatus::Deleted => Err(ApiKeyError::Deleted),
        }
    }

    pub fn id(&self) -> ApiKeyId {
        self.user_id
    }

    pub fn user(&self) -> String {
        self.user_name.to_string()
    }

    pub fn key(&self) -> String {
        self.api_key.to_string()
    }

    pub fn storage_key(&self) -> String {
        format!("{}-{}", self.id(), self.user())
    }
}

impl std::fmt::Display for ApiKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "ApiKey {{ id: {}, user: {}, key: {}, status: {:?} }}",
            self.user_id, self.user_name, self.api_key, self.status
        )
    }
}
