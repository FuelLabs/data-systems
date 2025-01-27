use fuel_streams_types::declare_integer_wrapper;

#[derive(thiserror::Error, Debug)]
pub enum ApiKeyUserIdError {
    #[error("Failed to parse to user_id: {0}")]
    InvalidFormat(String),
}

declare_integer_wrapper!(ApiKeyUserId, u32, ApiKeyUserIdError);
