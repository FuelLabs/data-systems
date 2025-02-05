use fuel_streams_core::{
    subjects::*,
    types::{StreamMessage, SubjectPayload},
};
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
    types::{
        DeliverPolicy,
        ServerRequest,
        ServerResponse,
        SubscriptionPayload,
    },
};
use crate::FuelNetwork;

#[derive(Debug, Clone)]
pub struct ConnectionOpts {
    pub network: FuelNetwork,
    pub api_key: Option<String>,
}

impl Default for ConnectionOpts {
    fn default() -> Self {
        Self {
            network: FuelNetwork::Local,
            api_key: None,
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
        message: &ServerRequest,
    ) -> Result<(), ClientError> {
        let mut write_guard = self.write_sink.write().await;
        let serialized = serde_json::to_vec(&message)?;
        write_guard
            .send(TungsteniteMessage::Binary(serialized.into()))
            .await?;
        Ok(())
    }

    async fn stream_with_message(
        &mut self,
        message: &ServerRequest,
    ) -> Result<
        impl Stream<Item = Result<StreamMessage, ClientError>> + '_ + Send + Unpin,
        ClientError,
    > {
        self.send_client_message(message).await?;
        let stream = self.read_stream.by_ref().filter_map(|msg| async {
            match msg {
                Ok(TungsteniteMessage::Binary(bin)) => {
                    match handle_binary_message(bin) {
                        Ok(Some(message)) => Some(Ok(message)),
                        Ok(None) => None,
                        Err(e) => Some(Err(e)),
                    }
                }
                Ok(TungsteniteMessage::Close(frame)) => {
                    Some(Err(ClientError::ConnectionClosed(
                        frame
                            .map(|f| f.to_string())
                            .unwrap_or_else(|| "Connection closed".to_string()),
                    )))
                }
                Ok(_) => None, // Ignore other message types
                Err(e) => Some(Err(ClientError::from(e))),
            }
        });

        Ok(Box::pin(stream))
    }

    pub async fn subscribe(
        &mut self,
        subject: impl IntoSubject + FromJsonString,
        deliver_policy: DeliverPolicy,
    ) -> Result<
        impl Stream<Item = Result<StreamMessage, ClientError>> + '_ + Send + Unpin,
        ClientError,
    > {
        let message = ServerRequest::Subscribe(SubscriptionPayload {
            deliver_policy,
            subject: subject.id().to_string(),
            params: subject.to_json(),
        });
        self.stream_with_message(&message).await
    }

    pub async fn unsubscribe(
        &self,
        subject: impl IntoSubject + FromJsonString,
        deliver_policy: DeliverPolicy,
    ) -> Result<(), ClientError> {
        let message = ServerRequest::Unsubscribe(SubscriptionPayload {
            subject: subject.id().to_string(),
            params: subject.to_json(),
            deliver_policy,
        });
        self.send_client_message(&message).await?;
        Ok(())
    }

    pub async fn subscribe_mult<T>(
        &mut self,
        subjects: Vec<SubjectPayload>,
        deliver_policy: DeliverPolicy,
    ) -> Result<
        impl Stream<Item = Result<StreamMessage, ClientError>> + '_ + Send + Unpin,
        ClientError,
    > {
        let sub_payloads: Vec<_> = subjects
            .into_iter()
            .map(|subject| SubscriptionPayload {
                deliver_policy,
                subject: subject.subject,
                params: subject.params,
            })
            .collect();
        let message = ServerRequest::Subscriptions(sub_payloads);
        self.stream_with_message(&message).await
    }
}

fn handle_binary_message(
    bin: tokio_tungstenite::tungstenite::Bytes,
) -> Result<Option<StreamMessage>, ClientError> {
    match serde_json::from_slice::<ServerResponse>(&bin) {
        Ok(ServerResponse::Response(response)) => Ok(Some(response)),
        Ok(ServerResponse::Error(e)) => Err(ClientError::Server(e)),
        Ok(_) => Ok(None),
        Err(e) => Err(ClientError::Server(e.to_string())),
    }
}
