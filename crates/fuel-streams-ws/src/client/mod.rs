use fuel_streams::{subjects::IntoSubject, types::FuelNetwork};
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
use tokio::sync::mpsc;
use tungstenite::{
    handshake::client::generate_key,
    protocol::Message,
    stream::MaybeTlsStream,
};
use url::Url;

use crate::server::{
    http::models::{LoginRequest, LoginResponse},
    ws::models::{
        ClientMessage,
        ServerMessage,
        SubscriptionPayload,
        SubscriptionType,
    },
};

#[derive(Debug)]
pub struct WebSocketClient {
    socket: Option<tungstenite::WebSocket<MaybeTlsStream<std::net::TcpStream>>>,
    jwt_token: String,
    ws_url: Url,
}

impl WebSocketClient {
    pub async fn new(
        network: FuelNetwork,
        username: &str,
        password: &str,
    ) -> anyhow::Result<Self> {
        let jwt_token = Self::fetch_jwt(network, username, password).await?;

        let ws_url = network
            .to_ws_url()
            .join("/api/v1/ws")
            .expect("valid relative path");

        Ok(Self {
            socket: None,
            jwt_token,
            ws_url,
        })
    }

    async fn fetch_jwt(
        network: FuelNetwork,
        username: &str,
        password: &str,
    ) -> anyhow::Result<String> {
        let client = HttpClient::new();
        let json_body = serde_json::to_string(&LoginRequest {
            username: username.to_string(),
            password: password.to_string(),
        })?;

        let api_url = network
            .to_web_url()
            .join("/api/v1/jwt")
            .expect("valid relative path");

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
            Err(anyhow::anyhow!(
                "Failed to fetch JWT: {}",
                response.status()
            ))
        }
    }

    pub fn connect(&mut self) -> anyhow::Result<()> {
        let host = self
            .ws_url
            .host_str()
            .ok_or(anyhow::anyhow!("Unparsable ws host url"))?;
        let request = tungstenite::handshake::client::Request::builder()
            .uri(self.ws_url.as_str())
            .header(AUTHORIZATION, format!("Bearer {}", self.jwt_token))
            .header(HOST, host)
            .header(UPGRADE, "websocket")
            .header(CONNECTION, "Upgrade")
            .header(SEC_WEBSOCKET_KEY, generate_key())
            .header(SEC_WEBSOCKET_VERSION, "13")
            .body(())
            .expect("Failed to build request");

        let (socket, _response) = tungstenite::connect(request)?;
        self.socket = Some(socket);

        Ok(())
    }

    fn send_client_message(
        &mut self,
        message: ClientMessage,
    ) -> anyhow::Result<()> {
        let socket = self
            .socket
            .as_mut()
            .ok_or_else(|| anyhow::anyhow!("Socket not connected"))?;
        let serialized = serde_json::to_vec(&message)?;
        socket.send(Message::Binary(serialized))?;
        Ok(())
    }

    pub fn subscribe(
        &mut self,
        subject: impl IntoSubject,
        from: Option<u64>,
        to: Option<u64>,
    ) -> anyhow::Result<()> {
        let message = ClientMessage::Subscribe(SubscriptionPayload {
            topic: SubscriptionType::Stream(subject.parse()),
            from,
            to,
        });
        self.send_client_message(message)
    }

    pub fn unsubscribe(
        &mut self,
        subject: impl IntoSubject,
        from: Option<u64>,
        to: Option<u64>,
    ) -> anyhow::Result<()> {
        let message = ClientMessage::Unsubscribe(SubscriptionPayload {
            topic: SubscriptionType::Stream(subject.parse()),
            from,
            to,
        });
        self.send_client_message(message)
    }

    pub fn listen(
        &mut self,
    ) -> anyhow::Result<mpsc::UnboundedReceiver<ServerMessage>> {
        let mut socket = self
            .socket
            .take()
            .ok_or_else(|| anyhow::anyhow!("Socket not connected"))?;

        let (tx, rx) = mpsc::unbounded_channel::<ServerMessage>();
        tokio::spawn(async move {
            loop {
                let msg = socket.read();
                match msg {
                    Ok(msg) => match msg {
                        Message::Text(text) => {
                            println!("Received text: {:?} bytes", text.len());
                        }
                        Message::Binary(bin) => {
                            println!("Received binary: {:?} bytes", bin.len());
                            let decoded =
                                serde_json::from_slice::<ServerMessage>(&bin)
                                    .unwrap();
                            println!("Decoded server message: {:?}", decoded);
                            if tx.send(decoded).is_err() {
                                break;
                            }
                        }
                        Message::Ping(ping) => {
                            println!("Received ping: {:?} bytes", ping.len());
                        }
                        Message::Pong(pong) => {
                            println!("Received pong: {:?} bytes", pong.len());
                        }
                        Message::Close(close) => {
                            let close_code = close
                                .as_ref()
                                .map(|c| c.code.to_string())
                                .unwrap_or_default();
                            let close_reason = close
                                .as_ref()
                                .map(|c| c.reason.to_string())
                                .unwrap_or_default();
                            println!("Received close with code: {:?} and reason: {:?}", close_code, close_reason);
                            break;
                        }
                        _ => {
                            eprintln!("Received unknown message type");
                            break;
                        }
                    },
                    Err(e) => {
                        eprintln!("Error reading message: {:?}", e);
                        break;
                    }
                }
            }
        });

        Ok(rx)
    }
}
