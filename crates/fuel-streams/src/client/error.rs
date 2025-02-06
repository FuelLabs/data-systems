use fuel_streams_store::record::RecordEntityError;

#[derive(Debug, thiserror::Error)]
pub enum ClientError {
    #[error(transparent)]
    JsonToString(#[from] serde_json::Error),
    #[error(transparent)]
    UrlParse(#[from] url::ParseError),
    #[error(transparent)]
    ApiResponse(#[from] reqwest::Error),
    #[error(transparent)]
    WebSocketConnect(#[from] tokio_tungstenite::tungstenite::Error),
    #[error(transparent)]
    InvalidHeaderValue(#[from] reqwest::header::InvalidHeaderValue),
    #[error(transparent)]
    RecordEntity(#[from] RecordEntityError),
    #[error("Failed to parse server message: {0}")]
    InvalidMessage(String),
    #[error("Failed to parse message data to {0}")]
    InvalidData(String),
    #[error("Server error: {0}")]
    Server(String),
    #[error("Failed to parse host from URL")]
    HostParseFailed,
    #[error("Missing api key")]
    MissingApiKey,
    #[error("Missing write sink")]
    MissingWriteSink,
    #[error("Missing read stream")]
    MissingReadStream,
    #[error("Missing WebSocket connection")]
    MissingWebSocketConnection,
    #[error("Failed when parsing MessageData from string: {0}")]
    MessageData(String),
    #[error("WebSocket connection closed unexpectedly at frame: {0}")]
    ConnectionClosed(String),
}
