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
    pub fn new(network: FuelNetwork) -> Self {
        Self {
            opts: ConnectionOpts {
                network,
                ..Default::default()
            },
        }
    }

    pub fn with_opts(opts: ConnectionOpts) -> Self {
        Self { opts }
    }

    pub fn with_api_key(&mut self, api_key: impl ToString) -> Self {
        Self {
            opts: ConnectionOpts {
                network: self.opts.network,
                api_key: Some(api_key.to_string()),
            },
        }
    }

    pub async fn connect(&mut self) -> Result<Connection, ClientError> {
        let api_key = self
            .opts
            .api_key
            .clone()
            .ok_or(ClientError::MissingApiKey)?;

        let subdirectory = format!("/api/v1/ws?api_key={}", api_key);
        let ws_url = self.opts.network.to_ws_url().join(&subdirectory)?;
        let host = ws_url
            .host_str()
            .ok_or_else(|| ClientError::HostParseFailed)?;

        let mut request = ws_url.as_str().into_client_request()?;
        let headers_map = request.headers_mut();
        headers_map.insert(HOST, host.parse()?);
        headers_map.insert(UPGRADE, "websocket".parse()?);
        headers_map.insert(CONNECTION, "Upgrade".parse()?);
        headers_map.insert(SEC_WEBSOCKET_KEY, generate_key().parse()?);
        headers_map.insert(SEC_WEBSOCKET_VERSION, "13".parse()?);
        Connection::new(request).await
    }
}
