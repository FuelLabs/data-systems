use std::sync::Arc;

use async_nats::{jetstream::Context, Client};
use async_trait::async_trait;
use futures::StreamExt;
use tracing::info;

use crate::{
    nats_metrics::{NatsHealthInfo, StreamInfo},
    Message,
    MessageBrokerError,
    MessageStream,
    Namespace,
    NatsOpts,
    NatsQueue,
};

#[derive(Debug)]
pub struct NatsMessage(pub async_nats::jetstream::Message);

#[derive(Debug, Clone)]
pub struct NatsMessageBroker {
    pub client: Client,
    pub jetstream: Context,
    pub namespace: Namespace,
    pub opts: NatsOpts,
}

impl NatsMessageBroker {
    async fn new(opts: &NatsOpts) -> Result<Self, MessageBrokerError> {
        let url = &opts.url();
        let client = opts.connect_opts().connect(url).await.map_err(|e| {
            MessageBrokerError::Connection(format!(
                "Failed to connect to NATS at {}: {}",
                url, e
            ))
        })?;
        info!("Connected to NATS server at {}", url);
        let jetstream = async_nats::jetstream::new(client.clone());
        Ok(Self {
            client,
            jetstream,
            namespace: opts.namespace.clone(),
            opts: opts.clone(),
        })
    }

    async fn with_url(url: &str) -> Result<Self, MessageBrokerError> {
        let opts = NatsOpts::new(url.to_string());
        Self::new(&opts).await
    }

    async fn with_namespace(
        url: &str,
        namespace: &str,
    ) -> Result<Self, MessageBrokerError> {
        let opts =
            crate::NatsOpts::new(url.to_string()).with_namespace(namespace);
        Self::new(&opts).await
    }

    pub async fn setup(
        url: &str,
        namespace: Option<&str>,
    ) -> Result<Arc<NatsMessageBroker>, MessageBrokerError> {
        let broker = match namespace {
            Some(namespace) => Self::with_namespace(url, namespace).await?,
            None => Self::with_url(url).await?,
        };
        broker.setup_queues().await?;
        Ok(broker.arc())
    }

    async fn setup_queues(&self) -> Result<(), MessageBrokerError> {
        let block_importer = NatsQueue::BlockImporter(self.arc());
        let block_retrier = NatsQueue::BlockRetrier(self.arc());
        block_importer.setup().await?;
        block_retrier.setup().await?;
        Ok(())
    }

    pub fn client(&self) -> Arc<Client> {
        Arc::new(self.client.to_owned())
    }

    pub fn arc(&self) -> Arc<Self> {
        Arc::new(self.clone())
    }

    pub fn namespace(&self) -> &Namespace {
        &self.namespace
    }

    pub fn is_connected(&self) -> bool {
        let state = self.client.connection_state();
        state == async_nats::connection::State::Connected
    }

    pub async fn publish(
        &self,
        topic: &str,
        payload: bytes::Bytes,
    ) -> Result<(), MessageBrokerError> {
        let subject = self.namespace().subject_name(topic);
        self.client
            .publish(subject, payload)
            .await
            .map_err(|e| MessageBrokerError::Publishing(e.to_string()))?;
        Ok(())
    }

    pub async fn subscribe(
        &self,
        topic: &str,
    ) -> Result<MessageStream, MessageBrokerError> {
        let subject = self.namespace().subject_name(topic);
        let stream = self
            .client
            .subscribe(subject)
            .await
            .map_err(|e| MessageBrokerError::Subscription(e.to_string()))?
            .map(|msg| Ok(bytes::Bytes::from(msg.payload.to_vec())));
        Ok(Box::new(stream))
    }

    pub async fn get_streams_info(
        &self,
    ) -> Result<Vec<StreamInfo>, MessageBrokerError> {
        let mut streams = self.jetstream.streams();
        let mut infos = vec![];
        while let Some(stream) = streams.next().await {
            let stream =
                stream.map_err(|e| MessageBrokerError::Setup(e.to_string()))?;
            infos.push(StreamInfo {
                stream_name: stream.config.name,
                state: stream.state.into(),
            });
        }
        Ok(infos)
    }

    pub async fn flush(&self) -> Result<(), MessageBrokerError> {
        self.client.flush().await.map_err(|e| {
            MessageBrokerError::Flush(format!(
                "Failed to flush NATS client: {}",
                e
            ))
        })?;
        Ok(())
    }

    pub async fn is_healthy(&self) -> bool {
        self.is_connected()
    }

    pub async fn get_health_info(
        &self,
        uptime_secs: u64,
    ) -> Result<serde_json::Value, MessageBrokerError> {
        let infos = self.get_streams_info().await?;
        let health_info = NatsHealthInfo {
            uptime_secs,
            is_healthy: self.is_healthy().await,
            streams_info: infos,
        };
        Ok(serde_json::to_value(health_info)?)
    }
}

#[async_trait]
impl Message for NatsMessage {
    fn payload(&self) -> Vec<u8> {
        self.0.payload.to_vec()
    }

    fn id(&self) -> String {
        self.0.subject.to_string()
    }

    async fn ack(&self) -> Result<(), MessageBrokerError> {
        self.0
            .ack()
            .await
            .map_err(|e| MessageBrokerError::Acknowledgment(e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use std::{
        sync::atomic::{AtomicUsize, Ordering},
        time::Duration,
    };

    use pretty_assertions::assert_eq;
    use rand::Rng;

    use super::*;
    use crate::NatsSubject;
    const NATS_URL: &str = "nats://localhost:4222";

    async fn setup_broker() -> Result<NatsMessageBroker, MessageBrokerError> {
        let opts = NatsOpts::new(NATS_URL.to_string())
            .with_rdn_namespace()
            .with_ack_wait(1);
        let broker = NatsMessageBroker::new(&opts).await?;
        broker.setup_queues().await?;
        Ok(broker)
    }

    #[tokio::test]
    async fn test_broker_connection() -> Result<(), MessageBrokerError> {
        let _broker = setup_broker().await?;
        Ok(())
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn test_pub_sub() -> Result<(), MessageBrokerError> {
        let broker = setup_broker().await?;
        let broker_clone = broker.clone();

        // Spawn a task to receive events
        let receiver = tokio::spawn(async move {
            let mut events = broker_clone.subscribe("test.topic").await?;
            tokio::time::timeout(Duration::from_secs(5), events.next())
                .await
                .map_err(|_| {
                    MessageBrokerError::Receiving(
                        "Timeout waiting for message".into(),
                    )
                })?
                .ok_or_else(|| {
                    MessageBrokerError::Receiving("No message received".into())
                })?
        });

        // Add a small delay to ensure subscriber is ready
        tokio::time::sleep(Duration::from_millis(100)).await;

        broker.publish("test.topic", vec![4, 5, 6].into()).await?;
        let result = receiver.await.expect("receiver task panicked")?;
        assert_eq!(result, bytes::Bytes::from(vec![4, 5, 6]));
        Ok(())
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn test_work_queue_batch_size_limiting(
    ) -> Result<(), MessageBrokerError> {
        let broker = setup_broker().await?;
        let queue = NatsQueue::BlockImporter(broker.arc());

        // Publish 3 messages
        queue
            .publish(&NatsSubject::BlockSubmitted(1_u64), vec![1])
            .await?;
        queue
            .publish(&NatsSubject::BlockSubmitted(2_u64), vec![2])
            .await?;
        queue
            .publish(&NatsSubject::BlockSubmitted(3_u64), vec![3])
            .await?;

        // Receive with batch size of 2
        let mut message_stream = queue.subscribe(2).await?;
        let mut received = Vec::new();
        while let Some(msg) = message_stream.next().await {
            let msg = msg?;
            received.push(msg.payload().to_vec());
            msg.ack().await?;
        }

        assert_eq!(
            received.len(),
            2,
            "Should only receive batch_size messages"
        );
        Ok(())
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn test_work_queue_unacked_message_redelivery(
    ) -> Result<(), MessageBrokerError> {
        let broker = setup_broker().await?;
        let queue = NatsQueue::BlockImporter(broker.arc());

        queue
            .publish(&NatsSubject::BlockSubmitted(1_u64), vec![1])
            .await?;

        {
            let mut message_stream = queue.subscribe(1).await?;
            let msg = message_stream.next().await.unwrap();
            assert!(msg.is_ok());
            let msg = msg.unwrap();
            assert_eq!(msg.payload(), &[1]);
        }

        // Message should be redelivered after ack wait of 1 second
        tokio::time::sleep(Duration::from_secs(2)).await;

        // Receive message again
        let mut message_stream = queue.subscribe(1).await?;
        let msg = message_stream.next().await.unwrap();
        assert!(msg.is_ok());
        Ok(())
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 3)]
    async fn test_work_queue_multiple_consumers(
    ) -> Result<(), MessageBrokerError> {
        let broker = setup_broker().await?;
        let processed = Arc::new(AtomicUsize::new(0));

        // Create message payloads
        let heights: Vec<u8> = (1..=10).collect(); // More messages to better test parallelism

        // Publish messages
        let queue = NatsQueue::BlockImporter(broker.arc());
        for height in &heights {
            queue
                .publish(&NatsSubject::BlockSubmitted(*height as u64), vec![
                    *height,
                ])
                .await?;
        }

        // Create consumer tasks
        let consumer_handles: Vec<_> = (0..3)
            .map(|_| {
                let queue = NatsQueue::BlockImporter(broker.arc());
                let processed = Arc::clone(&processed);

                tokio::spawn(async move {
                    let mut received = Vec::new();
                    let mut stream = queue.subscribe(1).await?;

                    while let Some(msg_result) = stream.next().await {
                        let msg = msg_result?;

                        // Simulate random processing time
                        let delay = rand::rng().random_range(100..500);
                        tokio::time::sleep(Duration::from_millis(delay)).await;

                        received.push(msg.payload().to_vec());
                        msg.ack().await?;
                        processed.fetch_add(1, Ordering::SeqCst);

                        // Break after processing some messages
                        if received.len() >= 3 {
                            break;
                        }
                    }

                    Ok::<Vec<Vec<u8>>, MessageBrokerError>(received)
                })
            })
            .collect();

        // Wait for all consumers to complete with timeout
        let results = tokio::time::timeout(
            Duration::from_secs(5),
            futures::future::join_all(consumer_handles),
        )
        .await
        .map_err(|_| {
            MessageBrokerError::Receiving("Consumers timed out".into())
        })?;

        // Verify results
        let total_processed = processed.load(Ordering::SeqCst);
        assert!(total_processed > 0, "Should have processed some messages");

        let all_received: Vec<u8> = results
            .into_iter()
            .filter_map(|r| r.ok())
            .flatten()
            .flatten()
            .flatten()
            .collect();

        assert!(!all_received.is_empty(), "Should have received messages");

        Ok(())
    }
}
