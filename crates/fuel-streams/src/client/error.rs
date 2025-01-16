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
}
