use fuel_streams::{
    logs::Log,
    subjects::IntoSubject,
    types::{Block, FuelNetwork, Input, Output, Receipt, Transaction},
    utxos::Utxo,
    Streamable,
};
use fuel_streams_storage::DeliverPolicy;
use futures_util::{
    stream::{SplitSink, SplitStream},
    SinkExt,
    StreamExt,
};
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
use tokio::sync::{mpsc, RwLock};
use tokio_tungstenite::{
    connect_async,
    tungstenite::{
        client::IntoClientRequest,
        handshake::client::generate_key,
        protocol::Message,
    },
    MaybeTlsStream,
};
use url::Url;

use crate::server::{
    http::models::{LoginRequest, LoginResponse},
    ws::{
        errors::WsSubscriptionError,
        models::{
            ClientMessage,
            ServerMessage,
            SubscriptionPayload,
            SubscriptionType,
        },
        socket::verify_and_extract_subject_name,
    },
};

#[derive(Debug)]
pub struct WebSocketClient {
    read_stream: Option<
        SplitStream<
            tokio_tungstenite::WebSocketStream<
                MaybeTlsStream<tokio::net::TcpStream>,
            >,
        >,
    >,
    write_sink: Option<
        RwLock<
            SplitSink<
                tokio_tungstenite::WebSocketStream<
                    MaybeTlsStream<tokio::net::TcpStream>,
                >,
                Message,
            >,
        >,
    >,
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

        let ws_url = network.to_ws_url().join("/api/v1/ws")?;

        Ok(Self {
            read_stream: None,
            write_sink: None,
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
            Err(anyhow::anyhow!(
                "Failed to fetch JWT: {}",
                response.status()
            ))
        }
    }

    pub async fn connect(&mut self) -> anyhow::Result<()> {
        let host = self
            .ws_url
            .host_str()
            .ok_or(anyhow::anyhow!("Unparsable ws host url"))?;

        let mut request = self.ws_url.as_str().into_client_request()?;
        let headers_map = request.headers_mut();
        headers_map.insert(
            AUTHORIZATION,
            format!("Bearer {}", self.jwt_token).parse()?,
        );
        headers_map.insert(HOST, host.parse()?);
        headers_map.insert(UPGRADE, "websocket".parse()?);
        headers_map.insert(CONNECTION, "Upgrade".parse().unwrap());
        headers_map.insert(SEC_WEBSOCKET_KEY, generate_key().parse()?);
        headers_map.insert(SEC_WEBSOCKET_VERSION, "13".parse()?);

        let (socket, _response) = connect_async(request).await?;
        let (write, read) = socket.split();

        self.read_stream = Some(read);
        self.write_sink = Some(RwLock::new(write));

        Ok(())
    }

    async fn send_client_message(
        &mut self,
        message: ClientMessage,
    ) -> anyhow::Result<()> {
        let write_sink = self
            .write_sink
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Socket not connected"))?;
        let mut write_guard = write_sink.write().await;
        let serialized = serde_json::to_vec(&message)?;
        write_guard.send(Message::Binary(serialized)).await?;
        Ok(())
    }

    pub async fn subscribe(
        &mut self,
        subject: impl IntoSubject,
        deliver_policy: DeliverPolicy,
    ) -> anyhow::Result<()> {
        let message = ClientMessage::Subscribe(SubscriptionPayload {
            topic: SubscriptionType::Stream(subject.parse()),
            deliver_policy,
        });
        self.send_client_message(message).await?;
        Ok(())
    }

    pub async fn unsubscribe(
        &mut self,
        subject: impl IntoSubject,
        deliver_policy: DeliverPolicy,
    ) -> anyhow::Result<()> {
        let message = ClientMessage::Unsubscribe(SubscriptionPayload {
            topic: SubscriptionType::Stream(subject.parse()),
            deliver_policy,
        });
        self.send_client_message(message).await?;
        Ok(())
    }

    pub async fn listen(
        &mut self,
    ) -> anyhow::Result<mpsc::UnboundedReceiver<ServerMessage>> {
        let read_stream = self
            .read_stream
            .take()
            .ok_or_else(|| anyhow::anyhow!("Socket not connected"))?;
        let (tx, rx) = mpsc::unbounded_channel::<ServerMessage>();
        // TODO: the reason for using this type of channel is due to the fact that Streamable cannot be currently
        // converted into a dynamic object trait, hence this approach of switching between types
        tokio::spawn(async move {
            let mut subscription_topic = String::new();
            let mut read_stream = read_stream;
            while let Some(Ok(msg)) = read_stream.next().await {
                match msg {
                    Message::Text(text) => {
                        println!("Received text: {:?} bytes", text.len());
                    }
                    Message::Binary(bin) => {
                        let server_message = match serde_json::from_slice::<
                            ServerMessage,
                        >(
                            &bin
                        ) {
                            Ok(server_message) => server_message,
                            Err(e) => {
                                eprintln!("Unparsable server message: {:?}", e);
                                continue;
                            }
                        };

                        match &server_message {
                            ServerMessage::Subscribed(sub) => {
                                println!(
                                    "Subscribed to topic: {:?}",
                                    sub.topic
                                );
                                let SubscriptionType::Stream(sub) = &sub.topic;
                                subscription_topic = sub.clone();
                            }
                            ServerMessage::Unsubscribed(sub) => {
                                println!(
                                    "Unsubscribed from topic: {:?}",
                                    sub.topic
                                );
                            }
                            ServerMessage::Update(update) => {
                                let _ = decode_print(
                                    &subscription_topic,
                                    update.clone(),
                                )
                                .ok();
                                // send server message over a channel to receivers
                                if tx.send(server_message).is_err() {
                                    break;
                                }
                            }
                            ServerMessage::Error(err) => {
                                println!("Received error from ws: {:?}", err);
                                break;
                            }
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
                        println!(
                            "Received close with code: {:?} and reason: {:?}",
                            close_code, close_reason
                        );
                        break;
                    }
                    _ => {
                        eprintln!("Received unknown message type");
                        break;
                    }
                }
            }
        });

        Ok(rx)
    }
}

pub fn decode_print(
    subject_wildcard: &str,
    s3_payload: Vec<u8>,
) -> Result<(), WsSubscriptionError> {
    let subject = verify_and_extract_subject_name(subject_wildcard)?;
    match subject.as_str() {
        Transaction::NAME => {
            let entity = serde_json::from_slice::<Transaction>(&s3_payload)
                .map_err(WsSubscriptionError::UnparsablePayload)?;
            println!("Update [{:?} bytes]-> {:?}", s3_payload.len(), entity);
        }
        Block::NAME => {
            let entity = serde_json::from_slice::<Block>(&s3_payload)
                .map_err(WsSubscriptionError::UnparsablePayload)?;
            println!("Update [{:?} bytes]-> {:?}", s3_payload.len(), entity);
        }
        Input::NAME => {
            let entity = serde_json::from_slice::<Input>(&s3_payload)
                .map_err(WsSubscriptionError::UnparsablePayload)?;
            println!("Update [{:?} bytes]-> {:?}", s3_payload.len(), entity);
        }
        Output::NAME => {
            let entity = serde_json::from_slice::<Output>(&s3_payload)
                .map_err(WsSubscriptionError::UnparsablePayload)?;
            println!("Update [{:?} bytes]-> {:?}", s3_payload.len(), entity);
        }
        Receipt::NAME => {
            let entity = serde_json::from_slice::<Receipt>(&s3_payload)
                .map_err(WsSubscriptionError::UnparsablePayload)?;
            println!("Update [{:?} bytes]-> {:?}", s3_payload.len(), entity);
        }
        Utxo::NAME => {
            let entity = serde_json::from_slice::<Utxo>(&s3_payload)
                .map_err(WsSubscriptionError::UnparsablePayload)?;
            println!("Update [{:?} bytes]-> {:?}", s3_payload.len(), entity);
        }
        Log::NAME => {
            let entity = serde_json::from_slice::<Log>(&s3_payload)
                .map_err(WsSubscriptionError::UnparsablePayload)?;
            println!("Update [{:?} bytes]-> {:?}", s3_payload.len(), entity);
        }
        _ => {
            eprintln!("Unknown entity {:?}", subject.to_string());
        }
    }
    Ok(())
}
