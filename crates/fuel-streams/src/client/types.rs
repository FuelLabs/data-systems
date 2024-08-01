use super::errors::ClientError;

#[derive(Debug, Clone, Eq, PartialEq, Default)]
pub enum ClientStatus {
    #[default]
    Pending,
    Connected,
}

pub type ConnectionResult<R> = Result<R, ClientError>;
