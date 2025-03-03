use fuel_streams_types::{declare_integer_wrapper, declare_string_wrapper};

#[derive(thiserror::Error, Debug)]
pub enum ApiKeyIdError {
    #[error("Failed to parse to user_id: {0}")]
    InvalidFormat(String),
}

#[derive(thiserror::Error, Debug)]
pub enum ApiKeyRoleIdError {
    #[error("Failed to parse to role_id: {0}")]
    InvalidFormat(String),
}

#[derive(thiserror::Error, Debug)]
pub enum SubscriptionCountError {
    #[error("Failed to parse to subscription_limit: {0}")]
    InvalidFormat(String),
}

#[derive(thiserror::Error, Debug)]
pub enum RateLimitPerMinuteError {
    #[error("Failed to parse to rate_limit_per_minute: {0}")]
    InvalidFormat(String),
}

#[derive(thiserror::Error, Debug)]
pub enum HistoricalLimitError {
    #[error("Failed to parse to historical_limit: {0}")]
    InvalidFormat(String),
}

declare_integer_wrapper!(ApiKeyId, u32, ApiKeyIdError);
declare_integer_wrapper!(ApiKeyRoleId, u32, ApiKeyRoleIdError);
declare_integer_wrapper!(SubscriptionCount, u32, SubscriptionCountError);
declare_integer_wrapper!(RateLimitPerMinute, u32, RateLimitPerMinuteError);
declare_integer_wrapper!(HistoricalLimit, u32, HistoricalLimitError);
declare_string_wrapper!(ApiKeyUserName);
declare_string_wrapper!(ApiKeyValue);
