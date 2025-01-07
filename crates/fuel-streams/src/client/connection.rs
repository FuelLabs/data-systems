use fuel_streams_core::subjects::IntoSubject;
use fuel_streams_store::db::Record;
use futures::{
    stream::{SplitSink, SplitStream},
    SinkExt,
    Stream,
    StreamExt,
};
use tokio::sync::RwLock;
use tokio_tungstenite::{
    connect_async,
    tungstenite::{http::Request, protocol::Message as TungsteniteMessage},
    MaybeTlsStream,
};

use super::{
    error::ClientError,
    types::{ClientMessage, DeliverPolicy, ServerMessage, SubscriptionPayload},
};
use crate::FuelNetwork;

#[derive(Debug, Clone)]
pub struct ConnectionOpts {
    pub network: FuelNetwork,
    pub username: String,
    pub password: String,
}

impl Default for ConnectionOpts {
    fn default() -> Self {
        Self {
            network: FuelNetwork::Local,
            username: "admin".to_string(),
            password: "admin".to_string(),
        }
    }
}

type ReadStream = SplitStream<
    tokio_tungstenite::WebSocketStream<MaybeTlsStream<tokio::net::TcpStream>>,
>;
type WriteSink = RwLock<
    SplitSink<
        tokio_tungstenite::WebSocketStream<
            MaybeTlsStream<tokio::net::TcpStream>,
        >,
        TungsteniteMessage,
    >,
>;

#[derive(Debug, Clone)]
pub struct Message<T> {
    pub subject: String,
    pub payload: T,
}

#[derive(Debug)]
pub struct Connection {
    pub read_stream: ReadStream,
    pub write_sink: WriteSink,
}

impl Connection {
    pub async fn new(req: Request<()>) -> Result<Self, ClientError> {
        let (socket, _response) = connect_async(req).await?;
        let (write, read) = socket.split();

        Ok(Self {
            read_stream: read,
            write_sink: RwLock::new(write),
        })
    }

    async fn send_client_message(
        &self,
        message: ClientMessage,
    ) -> Result<(), ClientError> {
        let mut write_guard = self.write_sink.write().await;
        let serialized = serde_json::to_vec(&message)?;
        write_guard
            .send(TungsteniteMessage::Binary(serialized.into()))
            .await?;
        Ok(())
    }

    pub async fn subscribe<T: Record>(
        &mut self,
        subject: impl IntoSubject,
        deliver_policy: DeliverPolicy,
    ) -> Result<impl Stream<Item = Message<T>> + '_ + Send + Unpin, ClientError>
    {
        let message = ClientMessage::Subscribe(SubscriptionPayload {
            wildcard: subject.parse(),
            deliver_policy,
        });
        self.send_client_message(message).await?;

        let stream = self.read_stream.by_ref().filter_map(|msg| async move {
            match msg {
                Ok(TungsteniteMessage::Binary(bin)) => {
                    match serde_json::from_slice::<ServerMessage>(&bin) {
                        Ok(ServerMessage::Response(value)) => {
                            match serde_json::from_value::<T>(value.payload) {
                                Ok(parsed) => Some(Message {
                                    subject: value.subject,
                                    payload: parsed,
                                }),
                                Err(e) => {
                                    eprintln!("Failed to parse value: {:?}", e);
                                    None
                                }
                            }
                        }
                        Ok(ServerMessage::Error(e)) => {
                            eprintln!("Server error: {}", e);
                            None
                        }
                        Ok(_) => None,
                        Err(e) => {
                            eprintln!("Unparsable server message: {:?}", e);
                            None
                        }
                    }
                }
                Ok(TungsteniteMessage::Close(_)) => None,
                Ok(msg) => {
                    println!("Received message: {:?}", msg);
                    None
                }
                Err(e) => {
                    eprintln!("WebSocket error: {:?}", e);
                    None
                }
            }
        });

        Ok(Box::pin(stream))
    }

    pub async fn unsubscribe<S: IntoSubject>(
        &self,
        subject: S,
        deliver_policy: DeliverPolicy,
    ) -> Result<(), ClientError> {
        let message = ClientMessage::Unsubscribe(SubscriptionPayload {
            wildcard: subject.parse(),
            deliver_policy,
        });
        self.send_client_message(message).await?;
        Ok(())
    }
}
