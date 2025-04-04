use std::sync::Arc;

use fuel_core_types::blockchain::SealedBlock;
use fuel_data_parser::DataEncoder;
use fuel_message_broker::{NatsMessageBroker, NatsQueue, NatsSubject};
use fuel_streams_core::types::FuelCoreLike;
use fuel_streams_domains::{Metadata, MsgPayload};
use fuel_web_utils::telemetry::Telemetry;

use crate::{error::PublishError, metrics::Metrics};

pub async fn publish_block(
    message_broker: &Arc<NatsMessageBroker>,
    fuel_core: &Arc<dyn FuelCoreLike>,
    sealed_block: &Arc<SealedBlock>,
    telemetry: &Arc<Telemetry<Metrics>>,
) -> Result<(), PublishError> {
    let metadata = Metadata::new(fuel_core, sealed_block);
    let payload = MsgPayload::new(fuel_core, sealed_block, &metadata).await?;
    let encoded = payload.encode().await?;
    let queue = NatsQueue::BlockImporter(message_broker.clone());
    let subject = NatsSubject::BlockSubmitted(payload.block_height().into());

    queue.publish(&subject, encoded.clone()).await?;
    if let Some(metrics) = telemetry.base_metrics() {
        metrics.update_publisher_success_metrics(
            &subject.to_string(&queue),
            encoded.len(),
        );
    }

    tracing::info!("New block submitted: {}", payload.block_height());
    Ok(())
}
