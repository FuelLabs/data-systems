use std::sync::Arc;

use fuel_streams_core::prelude::*;
use thiserror::Error;
use tokio::{sync::Semaphore, task::JoinHandle};

use crate::metrics::{publish_with_metrics, PublisherMetrics};

#[derive(Error, Debug)]
pub enum PublishError {
    #[error("Failed to publish to stream: {0}")]
    StreamPublish(String),
    #[error("Semaphore acquisition failed: {0}")]
    Semaphore(#[from] tokio::sync::AcquireError),
    #[error("Unknown error: {0}")]
    Unknown(String),
}

#[derive(Debug, Clone)]
pub struct PublishOpts {
    pub semaphore: Arc<Semaphore>,
    pub metrics: Arc<PublisherMetrics>,
    pub chain_id: Arc<ChainId>,
    pub block_producer: Arc<Address>,
    pub block_height: Arc<BlockHeight>,
}

// PublishPacket Struct
pub struct PublishPacket<S: Streamable + 'static> {
    subject: Arc<dyn IntoSubject>,
    wildcard: &'static str,
    payload: Arc<S>,
}

impl<T: Streamable + 'static> PublishPacket<T> {
    pub fn new(
        payload: &T,
        subject: Arc<dyn IntoSubject>,
        wildcard: &'static str,
    ) -> Self {
        Self {
            subject,
            wildcard,
            payload: Arc::new(payload.clone()), // Assuming T: Clone
        }
    }

    pub fn publish(
        &self,
        stream: Arc<Stream<T>>,
        opts: Arc<PublishOpts>,
    ) -> JoinHandle<Result<(), PublishError>> {
        let stream = Arc::clone(&stream);
        let opts = Arc::clone(&opts);
        let payload = Arc::clone(&self.payload);
        let subject = Arc::clone(&self.subject);
        let wildcard = self.wildcard;

        tokio::spawn(async move {
            let _permit = opts
                .semaphore
                .acquire()
                .await
                .map_err(PublishError::Semaphore)?;

            publish_with_metrics(
                stream.publish(&*subject, &payload),
                &opts.metrics,
                &opts.chain_id,
                &opts.block_producer,
                wildcard,
            )
            .await
            .map_err(|e| PublishError::StreamPublish(e.to_string()))
        })
    }
}
