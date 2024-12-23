use reqwest::{
    header::{
        ACCEPT,
        AUTHORIZATION,
        CONNECTION,
        CONTENT_TYPE,
        HOST,
        SEC_WEBSOCKET_KEY,
        SEC_WEBSOCKET_VERSION,
        UPGRADE,
    },
    Client as HttpClient,
};
use tokio_tungstenite::tungstenite::{
    client::IntoClientRequest,
    handshake::client::generate_key,
};

use super::{
    error::ClientError,
    Connection,
    ConnectionOpts,
    LoginRequest,
    LoginResponse,
};
use crate::FuelNetwork;

/// A client for connecting to the Fuel websocket server.
///
/// # Examples
///
/// ```no_run
/// use fuel_streams::{Client, FuelNetwork};
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     // Basic usage with default credentials
///     let mut client = Client::new(FuelNetwork::Local).await?;
///     let connection = client.connect().await?;
///
///     // Or with custom connection options
///     let client = Client::with_opts(ConnectionOpts {
///         network: FuelNetwork::Local,
///         username: "custom_user".to_string(),
///         password: "custom_pass".to_string(),
///     }).await?;
///     Ok(())
/// }
/// ```
#[derive(Debug, Clone)]
pub struct Client {
    pub opts: ConnectionOpts,
    pub jwt_token: Option<String>,
}

impl Client {
    /// Creates a new WebSocket client with default connection options for the specified network.
    pub async fn new(network: FuelNetwork) -> Result<Self, ClientError> {
        Self::with_opts(ConnectionOpts {
            network,
            ..Default::default()
        })
        .await
    }

    /// Creates a new WebSocket client with custom connection options.
    pub async fn with_opts(opts: ConnectionOpts) -> Result<Self, ClientError> {
        let jwt_token =
            Self::fetch_jwt(opts.network, &opts.username, &opts.password)
                .await?;
        Ok(Self {
            opts,
            jwt_token: Some(jwt_token),
        })
    }

    /// Establishes a WebSocket connection using the client's configuration.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use fuel_streams::{Client, FuelNetwork};
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let mut client = Client::new(FuelNetwork::Local).await?;
    ///     let connection = client.connect().await?;
    ///     Ok(())
    /// }
    /// ```
    pub async fn connect(&mut self) -> Result<Connection, ClientError> {
        let ws_url = self.opts.network.to_ws_url().join("/api/v1/ws")?;
        let host = ws_url
            .host_str()
            .ok_or_else(|| ClientError::HostParseFailed)?;

        let jwt_token =
            self.jwt_token.clone().ok_or(ClientError::MissingJwtToken)?;

        let bearer_token = format!("Bearer {}", jwt_token);
        let mut request = ws_url.as_str().into_client_request()?;
        let headers_map = request.headers_mut();
        headers_map.insert(AUTHORIZATION, bearer_token.parse()?);
        headers_map.insert(HOST, host.parse()?);
        headers_map.insert(UPGRADE, "websocket".parse()?);
        headers_map.insert(CONNECTION, "Upgrade".parse().unwrap());
        headers_map.insert(SEC_WEBSOCKET_KEY, generate_key().parse()?);
        headers_map.insert(SEC_WEBSOCKET_VERSION, "13".parse()?);
        Connection::new(request).await
    }

    /// Fetches a JWT token from the server for authentication.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use fuel_streams::{Client, FuelNetwork};
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let jwt = Client::fetch_jwt(
    ///         FuelNetwork::Local,
    ///         "admin",
    ///         "admin"
    ///     ).await?;
    ///
    ///     assert!(!jwt.is_empty());
    ///     Ok(())
    /// }
    /// ```
    async fn fetch_jwt(
        network: FuelNetwork,
        username: &str,
        password: &str,
    ) -> Result<String, ClientError> {
        let client = HttpClient::new();
        let json_body = serde_json::to_string(&LoginRequest {
            username: username.to_string(),
            password: password.to_string(),
        })?;

        let api_url = network.to_web_url().join("/api/v1/jwt")?;
        let response = client
            .get(api_url)
            .header(ACCEPT, "application/json")
            .header(CONTENT_TYPE, "application/json")
            .body(json_body)
            .send()
            .await?;

        if response.status().is_success() {
            let json_body = response.json::<LoginResponse>().await?;
            Ok(json_body.jwt_token)
        } else {
            Err(ClientError::ApiResponse(
                response.error_for_status_ref().unwrap_err(),
            ))
        }
    }

    /// Refreshes the JWT token and establishes a new WebSocket connection.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use fuel_streams::{Client, FuelNetwork};
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let mut client = Client::new(FuelNetwork::Local).await?;
    ///
    ///     // Refresh token and reconnect
    ///     let new_connection = client.refresh_jwt_and_connect().await?;
    ///     Ok(())
    /// }
    /// ```
    pub async fn refresh_jwt_and_connect(
        &mut self,
    ) -> Result<Connection, ClientError> {
        let jwt_token = Self::fetch_jwt(
            self.opts.network,
            &self.opts.username,
            &self.opts.password,
        )
        .await?;
        self.jwt_token = Some(jwt_token);
        self.connect().await
    }
}
