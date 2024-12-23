use displaydoc::Display as DisplayDoc;
use thiserror::Error;

#[derive(Debug, Error, DisplayDoc)]
pub enum ClientError {
    /// Failed to convert JSON to string: {0}
    JsonToString(#[from] serde_json::Error),

    /// Failed to parse URL: {0}
    UrlParse(#[from] url::ParseError),

    /// Failed to API response: {0}
    ApiResponse(#[from] reqwest::Error),

    /// Failed to connect to WebSocket: {0}
    WebSocketConnect(#[from] tokio_tungstenite::tungstenite::Error),

    /// Invalid header value: {0}
    InvalidHeaderValue(#[from] reqwest::header::InvalidHeaderValue),

    /// Failed to parse host from URL
    HostParseFailed,

    /// Missing JWT token
    MissingJwtToken,

    /// Missing write sink
    MissingWriteSink,

    /// Missing read stream
    MissingReadStream,

    /// Missing WebSocket connection
    MissingWebSocketConnection,
}
