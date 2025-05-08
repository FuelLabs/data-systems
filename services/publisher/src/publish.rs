use std::sync::{Arc, LazyLock};

use fuel_core_types::blockchain::SealedBlock;
use fuel_data_parser::DataEncoder;
use fuel_message_broker::{NatsMessageBroker, NatsQueue, NatsSubject};
use fuel_streams_core::types::FuelCoreLike;
use fuel_streams_domains::{Metadata, MsgPayload};
use fuel_streams_types::FuelCoreImporterResult;
use fuel_web_utils::telemetry::Telemetry;

use crate::{error::PublishError, metrics::Metrics};

pub static ONLY_EVENTS: LazyLock<bool> = LazyLock::new(|| {
    dotenvy::var("ONLY_EVENTS")
        .ok()
        .and_then(|val| val.parse().ok())
        .unwrap_or(false)
});

pub async fn publish_block(
    message_broker: &Arc<NatsMessageBroker>,
    fuel_core: &Arc<dyn FuelCoreLike>,
    sealed_block: &Arc<SealedBlock>,
    telemetry: &Arc<Telemetry<Metrics>>,
    importer_result: Option<&FuelCoreImporterResult>,
) -> Result<(), PublishError> {
    let metadata = Metadata::new(fuel_core, sealed_block);
    let events = importer_result
        .as_ref()
        .map(|i| i.events.to_owned())
        .unwrap_or_default();
    let payload =
        MsgPayload::new(fuel_core, sealed_block, &metadata, events).await?;
    let encoded = payload.encode_json()?;

    // Publish block submitted event
    // TODO: This is temporary until we have events synced
    if !*ONLY_EVENTS {
        let importer_queue = NatsQueue::BlockImporter(message_broker.clone());
        let subject =
            NatsSubject::BlockSubmitted(payload.block_height().into());
        importer_queue.publish(&subject, encoded.clone()).await?;

        if let Some(metrics) = telemetry.base_metrics() {
            metrics.update_publisher_success_metrics(
                &subject.to_string(&importer_queue),
                encoded.len(),
            );
        }
    }

    // Publish block event
    let event_queue = NatsQueue::BlockEvent(message_broker.clone());
    let event_subject = NatsSubject::BlockEvent(payload.block_height().into());
    event_queue.publish(&event_subject, encoded.clone()).await?;

    tracing::info!("New block submitted: {}", payload.block_height());
    Ok(())
}
