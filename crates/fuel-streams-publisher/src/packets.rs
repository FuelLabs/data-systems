use std::sync::Arc;

use fuel_streams_core::prelude::*;
use thiserror::Error;
use tokio::{sync::Semaphore, task::JoinHandle};

use crate::telemetry::Telemetry;

#[derive(Error, Debug)]
pub enum PublishError {
    #[error("Failed to publish to stream: {0}")]
    StreamPublish(String),
    #[error("Semaphore acquisition failed: {0}")]
    Semaphore(#[from] tokio::sync::AcquireError),
    #[error("Unknown error: {0}")]
    Unknown(String),
}

#[derive(Clone)]
pub struct PublishOpts {
    pub semaphore: Arc<Semaphore>,
    pub chain_id: Arc<ChainId>,
    pub block_producer: Arc<Address>,
    pub block_height: Arc<BlockHeight>,
    pub telemetry: Arc<Telemetry>,
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
        let telemetry = Arc::clone(&opts.telemetry);

        tokio::spawn(async move {
            let _permit = opts
                .semaphore
                .acquire()
                .await
                .map_err(PublishError::Semaphore)?;

            match stream.publish(&*subject, &payload).await {
                Ok(published_data_size) => {
                    telemetry.log_info(&format!(
                        "Successfully published for stream: {}",
                        wildcard
                    ));
                    telemetry.update_publisher_success_metrics(
                        wildcard,
                        published_data_size,
                        &opts.chain_id,
                        &opts.block_producer,
                    );

                    Ok(())
                }
                Err(e) => {
                    telemetry.log_error(&e.to_string());
                    telemetry.update_publisher_error_metrics(
                        wildcard,
                        &opts.chain_id,
                        &opts.block_producer,
                        &e.to_string(),
                    );

                    Err(PublishError::StreamPublish(e.to_string()))
                }
            }
        })
    }
}
