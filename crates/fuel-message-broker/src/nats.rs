use std::{sync::Arc, time::Duration};

use async_nats::{
    jetstream::{
        consumer::{pull::Config as ConsumerConfig, AckPolicy, PullConsumer},
        context::Publish,
        stream::{Config as StreamConfig, RetentionPolicy},
        Context,
    },
    Client,
};
use async_trait::async_trait;
use futures::StreamExt;
use tracing::info;

use crate::{
    nats_metrics::StreamInfo,
    Message,
    MessageBlockStream,
    MessageBroker,
    MessageBrokerError,
    MessageStream,
    Namespace,
    NatsOpts,
};

#[derive(Debug)]
pub struct NatsMessage(async_nats::jetstream::Message);

#[derive(Debug, Clone)]
pub struct NatsMessageBroker {
    pub client: Client,
    pub jetstream: Context,
    pub namespace: Namespace,
    pub opts: NatsOpts,
}

impl NatsMessageBroker {
    const BLOCKS_STREAM: &'static str = "block_importer";
    const BLOCKS_SUBJECT: &'static str = "block_submitted";

    pub async fn new(opts: &NatsOpts) -> Result<Self, MessageBrokerError> {
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

    fn stream_name(&self) -> String {
        self.namespace().queue_name(Self::BLOCKS_STREAM)
    }

    fn consumer_name(&self) -> String {
        format!("{}_consumer", self.stream_name())
    }

    fn blocks_subject(&self) -> String {
        self.namespace().subject_name(Self::BLOCKS_SUBJECT)
    }

    async fn get_blocks_stream(
        &self,
    ) -> Result<async_nats::jetstream::stream::Stream, MessageBrokerError> {
        let subject_name = format!("{}.>", self.blocks_subject());
        let stream_name = self.stream_name();
        let stream = self
            .jetstream
            .get_or_create_stream(StreamConfig {
                name: stream_name,
                subjects: vec![subject_name],
                retention: RetentionPolicy::WorkQueue,
                duplicate_window: Duration::from_secs(1),
                allow_rollup: true,
                ..Default::default()
            })
            .await
            .map_err(|e| MessageBrokerError::Setup(e.to_string()))?;
        Ok(stream)
    }

    async fn get_blocks_consumer(
        &self,
    ) -> Result<PullConsumer, MessageBrokerError> {
        let consumer_name = self.consumer_name();
        let stream = self.get_blocks_stream().await?;
        stream
            .get_or_create_consumer(&consumer_name, ConsumerConfig {
                durable_name: Some(consumer_name.to_string()),
                ack_policy: AckPolicy::Explicit,
                ack_wait: Duration::from_secs(self.opts.ack_wait_secs),
                ..Default::default()
            })
            .await
            .map_err(|e| MessageBrokerError::Setup(e.to_string()))
    }

    pub async fn get_stream_info(&self) -> Vec<StreamInfo> {
        // let streams = self.
        todo!()
    }

    pub fn arc(&self) -> Arc<Self> {
        Arc::new(self.clone())
    }
}

#[async_trait]
impl Message for NatsMessage {
    fn payload(&self) -> Vec<u8> {
        self.0.payload.to_vec()
    }

    async fn ack(&self) -> Result<(), MessageBrokerError> {
        self.0
            .ack()
            .await
            .map_err(|e| MessageBrokerError::Acknowledgment(e.to_string()))
    }
}

#[async_trait]
impl MessageBroker for NatsMessageBroker {
    fn namespace(&self) -> &Namespace {
        &self.namespace
    }

    fn is_connected(&self) -> bool {
        let state = self.client.connection_state();
        state == async_nats::connection::State::Connected
    }

    async fn setup(&self) -> Result<(), MessageBrokerError> {
        let _ = self.get_blocks_stream().await?;
        Ok(())
    }

    async fn publish_block(
        &self,
        id: String,
        payload: Vec<u8>,
    ) -> Result<(), MessageBrokerError> {
        let subject = format!("{}.{}", self.blocks_subject(), id);
        let payload_id = format!("{}.block_{}", self.namespace(), id);
        let publish = Publish::build()
            .message_id(payload_id)
            .payload(payload.into());
        self.jetstream
            .send_publish(subject, publish)
            .await
            .map_err(|e| MessageBrokerError::Publishing(e.to_string()))?
            .await
            .map_err(|e| MessageBrokerError::Publishing(e.to_string()))?;

        Ok(())
    }

    async fn receive_blocks_stream(
        &self,
        batch_size: usize,
    ) -> Result<MessageBlockStream, MessageBrokerError> {
        let consumer = self.get_blocks_consumer().await?;
        let stream = consumer
            .fetch()
            .max_messages(batch_size)
            .messages()
            .await
            .map_err(|e| MessageBrokerError::Receiving(e.to_string()))?
            .filter_map(|msg| async {
                msg.ok()
                    .map(|m| Ok(Box::new(NatsMessage(m)) as Box<dyn Message>))
            })
            .boxed();
        Ok(Box::new(stream))
    }

    async fn publish_event(
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

    async fn subscribe_to_events(
        &self,
        topic: &str,
    ) -> Result<MessageStream, MessageBrokerError> {
        let subject = self.namespace().subject_name(topic);
        let stream = self
            .client
            .subscribe(subject)
            .await
            .map_err(|e| MessageBrokerError::Subscription(e.to_string()))?
            .map(|msg| Ok(msg.payload.to_vec()));
        Ok(Box::new(stream))
    }

    async fn flush(&self) -> Result<(), MessageBrokerError> {
        self.client.flush().await.map_err(|e| {
            MessageBrokerError::Flush(format!(
                "Failed to flush NATS client: {}",
                e
            ))
        })?;
        Ok(())
    }

    async fn is_healthy(&self) -> bool {
        self.is_connected()
    }

    async fn get_health_info(&self) -> serde_json::Value {
        serde_json::json!({})
    }
}

#[cfg(test)]
mod tests {
    use rand::Rng;

    use super::*;
    const NATS_URL: &str = "nats://localhost:4222";

    async fn setup_broker() -> Result<NatsMessageBroker, MessageBrokerError> {
        let opts = NatsOpts::new(NATS_URL.to_string())
            .with_rdn_namespace()
            .with_ack_wait(1);
        let broker = NatsMessageBroker::new(&opts).await?;
        broker.setup().await?;
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
            let mut events =
                broker_clone.subscribe_to_events("test.topic").await?;

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

        broker
            .publish_event("test.topic", vec![4, 5, 6].into())
            .await?;
        let result = receiver.await.expect("receiver task panicked")?;
        assert_eq!(result, vec![4, 5, 6]);
        Ok(())
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn test_work_queue() -> Result<(), MessageBrokerError> {
        let broker = setup_broker().await?;
        let broker_clone = broker.clone();

        // Spawn a task to receive events
        let receiver = tokio::spawn(async move {
            let mut messages = Vec::new();
            let mut stream = broker_clone.receive_blocks_stream(3).await?;
            while let Some(msg) = stream.next().await {
                let msg = msg?;
                messages.push(msg);
                if messages.len() >= 3 {
                    break;
                }
            }
            Ok::<Vec<Box<dyn Message>>, MessageBrokerError>(messages)
        });

        // Publish multiple messages
        broker.publish_block("1".to_string(), vec![1, 2, 3]).await?;
        broker.publish_block("2".to_string(), vec![4, 5, 6]).await?;
        broker.publish_block("3".to_string(), vec![7, 8, 9]).await?;

        // Wait for receiver and check results
        let messages = receiver.await.expect("receiver task panicked")?;
        assert_eq!(messages.len(), 3, "Expected to receive 3 messages");
        assert_eq!(messages[0].payload(), &[1, 2, 3]);
        assert_eq!(messages[1].payload(), &[4, 5, 6]);
        assert_eq!(messages[2].payload(), &[7, 8, 9]);

        // Acknowledge all messages
        for msg in messages {
            msg.ack().await?;
        }

        Ok(())
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn test_work_queue_batch_size_limiting(
    ) -> Result<(), MessageBrokerError> {
        let broker = setup_broker().await?;

        // Publish 3 messages
        broker.publish_block("1".to_string(), vec![1]).await?;
        broker.publish_block("2".to_string(), vec![2]).await?;
        broker.publish_block("3".to_string(), vec![3]).await?;

        // Receive with batch size of 2
        let mut stream = broker.receive_blocks_stream(2).await?;
        let mut received = Vec::new();
        while let Some(msg) = stream.next().await {
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
        broker.publish_block("1".to_string(), vec![1]).await?;

        {
            let mut stream = broker.receive_blocks_stream(1).await?;
            let msg = stream.next().await.unwrap();
            assert!(msg.is_ok());
            let msg = msg.unwrap();
            assert_eq!(msg.payload(), &[1]);
        }

        // Message should be redelivered after ack wait of 1 second
        tokio::time::sleep(Duration::from_secs(2)).await;

        // Receive message again
        let mut stream = broker.receive_blocks_stream(1).await?;
        let msg = stream.next().await.unwrap();
        assert!(msg.is_ok());
        Ok(())
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn test_work_queue_multiple_consumers(
    ) -> Result<(), MessageBrokerError> {
        let broker = setup_broker().await?;
        let broker1 = broker.clone();
        let broker2 = broker.clone();
        let broker3 = broker.clone();

        // Spawn three consumer tasks
        let consumer1 = tokio::spawn(async move {
            let mut stream = broker1.receive_blocks_stream(1).await?;
            let msg = stream.next().await.ok_or_else(|| {
                MessageBrokerError::Receiving("No message received".into())
            })??;
            msg.ack().await?;
            Ok::<Vec<u8>, MessageBrokerError>(msg.payload().to_vec())
        });

        let consumer2 = tokio::spawn(async move {
            let mut stream = broker2.receive_blocks_stream(1).await?;
            let msg = stream.next().await.ok_or_else(|| {
                MessageBrokerError::Receiving("No message received".into())
            })??;
            msg.ack().await?;
            Ok::<Vec<u8>, MessageBrokerError>(msg.payload().to_vec())
        });

        let consumer3 = tokio::spawn(async move {
            let mut stream = broker3.receive_blocks_stream(1).await?;
            let msg = stream.next().await.ok_or_else(|| {
                MessageBrokerError::Receiving("No message received".into())
            })??;
            msg.ack().await?;
            Ok::<Vec<u8>, MessageBrokerError>(msg.payload().to_vec())
        });

        let heights = (0..3).map(|_| random_height() as u8).collect::<Vec<_>>();

        // Publish three messages
        broker
            .publish_block(heights[0].to_string(), vec![heights[0]])
            .await?;
        broker
            .publish_block(heights[1].to_string(), vec![heights[1]])
            .await?;
        broker
            .publish_block(heights[2].to_string(), vec![heights[2]])
            .await?;

        // Collect results from all consumers
        let msg1 = consumer1.await.expect("consumer1 task panicked")?;
        let msg2 = consumer2.await.expect("consumer2 task panicked")?;
        let msg3 = consumer3.await.expect("consumer3 task panicked")?;

        // Verify that each consumer got a different message
        let mut received = vec![msg1[0], msg2[0], msg3[0]];
        let mut heights = heights.clone();
        received.sort();
        heights.sort();
        assert_eq!(
            received, heights,
            "Consumers should receive all messages, regardless of order"
        );

        Ok(())
    }

    fn random_height() -> u32 {
        rand::thread_rng().gen_range(1..1000)
    }
}
