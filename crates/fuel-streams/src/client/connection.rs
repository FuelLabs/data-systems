use fuel_streams_core::{subjects::IntoSubject, Streamable};
use futures::{
    stream::{SplitSink, SplitStream},
    SinkExt,
    Stream,
    StreamExt,
};
use tokio::sync::RwLock;
use tokio_tungstenite::{
    connect_async,
    tungstenite::{http::Request, protocol::Message},
    MaybeTlsStream,
};

use super::{
    error::ClientError,
    types::{ClientMessage, DeliverPolicy, ServerMessage, SubscriptionPayload},
};
use crate::FuelNetwork;

/// Connection options for establishing a WebSocket connection.
///
/// # Examples
///
/// ```
/// use fuel_streams::{ConnectionOpts, FuelNetwork};
///
/// // Create connection options with custom values
/// let opts = ConnectionOpts {
///     network: FuelNetwork::Local,
///     username: "admin".to_string(),
///     password: "admin".to_string(),
/// };
///
/// // Or use the default options
/// let default_opts = ConnectionOpts::default();
/// assert_eq!(default_opts.username, "admin");
/// assert_eq!(default_opts.password, "admin");
/// assert!(matches!(default_opts.network, FuelNetwork::Local));
/// ```
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
        Message,
    >,
>;

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
    /// Sends a client message through the WebSocket connection.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use fuel_streams::{Client, ConnectionOpts, FuelNetwork};
    /// use sv_webserver::server::ws::models::ClientMessage;
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let mut client = Client::new(FuelNetwork::Local).await?;
    ///     let connection = client.connect().await?;
    ///
    ///     let message = ClientMessage::Ping;
    ///     connection.send_client_message(message).await?;
    ///     Ok(())
    /// }
    /// ```
    async fn send_client_message(
        &self,
        message: ClientMessage,
    ) -> Result<(), ClientError> {
        let mut write_guard = self.write_sink.write().await;
        let serialized = serde_json::to_vec(&message)?;
        write_guard.send(Message::Binary(serialized.into())).await?;
        Ok(())
    }

    /// Subscribes to a subject and returns a stream of messages.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use fuel_streams::{Client, ConnectionOpts, FuelNetwork};
    /// use sv_webserver::server::ws::models::{DeliverPolicy, ServerMessage};
    /// use std::sync::Arc;
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let mut client = Client::new(FuelNetwork::Local).await?;
    ///     let mut connection = client.connect().await?;
    ///
    ///     let subject = Arc::new("example.subject");
    ///     let mut stream = connection.subscribe::<ServerMessage>(
    ///         subject,
    ///         DeliverPolicy::All
    ///     ).await?;
    ///
    ///     // Process messages from the stream
    ///     while let Some(message) = stream.next().await {
    ///         println!("Received: {:?}", message);
    ///     }
    ///     Ok(())
    /// }
    /// ```
    pub async fn subscribe<T: Streamable>(
        &mut self,
        subject: impl IntoSubject,
        deliver_policy: DeliverPolicy,
    ) -> Result<impl Stream<Item = T> + '_ + Send + Unpin, ClientError> {
        let message = ClientMessage::Subscribe(SubscriptionPayload {
            wildcard: subject.parse(),
            deliver_policy,
        });
        self.send_client_message(message).await?;

        let stream = self.read_stream.by_ref().filter_map(|msg| async move {
            match msg {
                Ok(Message::Binary(bin)) => {
                    match serde_json::from_slice::<ServerMessage>(&bin) {
                        Ok(ServerMessage::Response(value)) => {
                            match serde_json::from_value::<T>(value) {
                                Ok(parsed) => Some(parsed),
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
                Ok(Message::Close(_)) => None,
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

    /// Unsubscribes from a subject.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use fuel_streams::{Client, ConnectionOpts, FuelNetwork};
    /// use sv_webserver::server::ws::models::DeliverPolicy;
    /// use std::sync::Arc;
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let mut client = Client::new(FuelNetwork::Local).await?;
    ///     let connection = client.connect().await?;
    ///
    ///     let subject = Arc::new("example.subject");
    ///     connection.unsubscribe(subject, DeliverPolicy::All).await?;
    ///     Ok(())
    /// }
    /// ```
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
