use std::{sync::Arc, time::Duration};

use async_nats::jetstream::{
    consumer::{pull::Config as ConsumerConfig, AckPolicy, PullConsumer},
    stream::{Config as StreamConfig, RetentionPolicy},
};
use futures::StreamExt;

use crate::{MessageBrokerError, NatsMessageBroker};

pub enum NatsSubject {
    BlockSubmitted(u64),
    BlockFailed(u64),
    BlockSuccess(u64),
}

impl NatsSubject {
    pub fn to_string(&self, queue: &NatsQueue) -> String {
        let queue_name = queue.queue_name();
        match self {
            NatsSubject::BlockSubmitted(height) => {
                format!("{queue_name}.block_submitted.{height}")
            }
            NatsSubject::BlockFailed(id) => {
                format!("{queue_name}.block_failed.{id}")
            }
            NatsSubject::BlockSuccess(id) => {
                format!("{queue_name}.block_success.{id}")
            }
        }
    }

    fn to_id(&self, queue: &NatsQueue) -> String {
        self.to_string(queue).replace(".", "_")
    }
}

pub enum NatsQueue {
    BlockImporter(Arc<NatsMessageBroker>),
    BlockRetrier(Arc<NatsMessageBroker>),
}

impl NatsQueue {
    fn broker(&self) -> &NatsMessageBroker {
        match self {
            NatsQueue::BlockImporter(broker) => broker,
            NatsQueue::BlockRetrier(broker) => broker,
        }
    }

    fn queue_name(&self) -> String {
        let value = match self {
            NatsQueue::BlockImporter(_) => "block_importer",
            NatsQueue::BlockRetrier(_) => "block_retrier",
        };
        self.broker().namespace().queue_name(value)
    }

    fn subjects(&self) -> Vec<String> {
        let queue_name = self.queue_name();
        match self {
            NatsQueue::BlockImporter(_) => {
                vec![format!("{queue_name}.block_submitted.>")]
            }
            NatsQueue::BlockRetrier(_) => {
                vec![format!("{queue_name}.block_failed.>")]
            }
        }
    }

    fn consumer_name(&self) -> String {
        format!("{}_consumer", self.queue_name())
    }

    pub async fn get_or_create_stream(
        &self,
    ) -> Result<async_nats::jetstream::stream::Stream, MessageBrokerError> {
        let broker = self.broker();
        let subjects = self.subjects();
        let name = self.queue_name();
        broker
            .jetstream
            .get_or_create_stream(StreamConfig {
                name,
                subjects,
                retention: RetentionPolicy::WorkQueue,
                duplicate_window: Duration::from_secs(1),
                allow_direct: true,
                ..Default::default()
            })
            .await
            .map_err(|e| MessageBrokerError::Setup(e.to_string()))
    }

    pub async fn get_or_create_consumer(
        &self,
    ) -> Result<PullConsumer, MessageBrokerError> {
        let stream = self.get_or_create_stream().await?;
        let consumer_name = self.consumer_name();
        let broker = self.broker();
        let mut config = ConsumerConfig {
            durable_name: Some(consumer_name.to_string()),
            ack_policy: AckPolicy::Explicit,
            ..Default::default()
        };

        if let Some(ack_wait) = broker.opts.ack_wait_secs {
            config.ack_wait = Duration::from_secs(ack_wait);
        }

        stream
            .get_or_create_consumer(&consumer_name, config)
            .await
            .map_err(|e| MessageBrokerError::Setup(e.to_string()))
    }

    pub async fn publish<T: Into<bytes::Bytes>>(
        &self,
        subject: &NatsSubject,
        payload: T,
    ) -> Result<(), MessageBrokerError> {
        let broker = self.broker();
        let subject_str = subject.to_string(self);
        let subject_id = subject.to_id(self);
        let publish = async_nats::jetstream::context::Publish::build()
            .message_id(subject_id)
            .payload(payload.into());

        broker
            .jetstream
            .send_publish(subject_str, publish)
            .await
            .map_err(|e| MessageBrokerError::Publishing(e.to_string()))?
            .await
            .map_err(|e| MessageBrokerError::Publishing(e.to_string()))?;

        Ok(())
    }

    pub async fn subscribe(
        &self,
        batch_size: usize,
    ) -> Result<crate::MessageBlockStream, MessageBrokerError> {
        let consumer = self.get_or_create_consumer().await?;
        let stream = consumer
            .fetch()
            .max_messages(batch_size)
            .messages()
            .await
            .map_err(|e| MessageBrokerError::Receiving(e.to_string()))?
            .filter_map(|msg| async {
                msg.ok().map(|m| {
                    Ok(Box::new(crate::NatsMessage(m))
                        as Box<dyn crate::Message>)
                })
            })
            .boxed();
        Ok(Box::new(stream))
    }

    pub async fn setup(&self) -> Result<(), MessageBrokerError> {
        let _ = self.get_or_create_stream().await?;
        let _ = self.get_or_create_consumer().await?;
        Ok(())
    }
}
