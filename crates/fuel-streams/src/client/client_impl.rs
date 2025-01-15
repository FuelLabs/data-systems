use reqwest::header::{
    CONNECTION,
    HOST,
    SEC_WEBSOCKET_KEY,
    SEC_WEBSOCKET_VERSION,
    UPGRADE,
};
use tokio_tungstenite::tungstenite::{
    client::IntoClientRequest,
    handshake::client::generate_key,
};

use super::{error::ClientError, Connection, ConnectionOpts};
use crate::FuelNetwork;

#[derive(Debug, Clone)]
pub struct Client {
    pub opts: ConnectionOpts,
}

impl Client {
    pub async fn new(network: FuelNetwork) -> Result<Self, ClientError> {
        Self::with_opts(ConnectionOpts {
            network,
            ..Default::default()
        })
        .await
    }

    pub async fn with_opts(opts: ConnectionOpts) -> Result<Self, ClientError> {
        Ok(Self { opts })
    }

    pub async fn connect(&mut self) -> Result<Connection, ClientError> {
        let jwt_token = self
            .opts
            .api_key
            .clone()
            .ok_or(ClientError::MissingApiKey)?;

        let subdirectory = format!("/api/v1/ws?api_key={}", jwt_token);
        let ws_url = self.opts.network.to_ws_url().join(&subdirectory)?;
        let host = ws_url
            .host_str()
            .ok_or_else(|| ClientError::HostParseFailed)?;

        let mut request = ws_url.as_str().into_client_request()?;
        let headers_map = request.headers_mut();
        headers_map.insert(HOST, host.parse()?);
        headers_map.insert(UPGRADE, "websocket".parse()?);
        headers_map.insert(CONNECTION, "Upgrade".parse().unwrap());
        headers_map.insert(SEC_WEBSOCKET_KEY, generate_key().parse()?);
        headers_map.insert(SEC_WEBSOCKET_VERSION, "13".parse()?);
        Connection::new(request).await
    }
}
