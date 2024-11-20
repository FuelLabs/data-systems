use std::sync::Arc;

use fuel_streams_core::prelude::*;
use fuel_streams_types::Consensus;
use tokio::{sync::Semaphore, task::JoinHandle};

use crate::telemetry::Telemetry;

#[derive(Clone)]
pub struct PublishOpts {
    pub semaphore: Arc<Semaphore>,
    pub chain_id: Arc<ChainId>,
    pub block_producer: Arc<Address>,
    pub block_height: Arc<BlockHeight>,
    pub telemetry: Arc<Telemetry>,
    pub consensus: Arc<Consensus>,
}

// PublishPacket Struct
pub struct PublishPacket<S: Streamable> {
    subject: Arc<dyn IntoSubject>,
    payload: Arc<S>,
}

impl<S: Streamable + 'static> PublishPacket<S> {
    pub fn new(payload: &S, subject: Arc<dyn IntoSubject>) -> Self {
        Self {
            subject,
            payload: Arc::new(payload.clone()), // Assuming T: Clone
        }
    }

    pub fn publish(
        &self,
        stream: Arc<Stream<S>>,
        opts: &Arc<PublishOpts>,
    ) -> JoinHandle<anyhow::Result<()>> {
        let stream = Arc::clone(&stream);
        let opts = Arc::clone(opts);
        let payload = Arc::clone(&self.payload);
        let subject = Arc::clone(&self.subject);
        let telemetry = Arc::clone(&opts.telemetry);
        let wildcard = self.subject.wildcard();

        tokio::spawn(async move {
            let _permit = opts.semaphore.acquire().await?;

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

                    anyhow::bail!("Failed to publish: {}", e.to_string())
                }
            }
        })
    }
}
