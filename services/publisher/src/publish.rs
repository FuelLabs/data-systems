use std::sync::Arc;

use fuel_core_types::blockchain::SealedBlock;
use fuel_message_broker::MessageBroker;
use fuel_streams_core::types::FuelCoreLike;
use fuel_streams_domains::{Metadata, MsgPayload};
use fuel_streams_store::record::DataEncoder;
use fuel_web_utils::telemetry::Telemetry;

use crate::{error::PublishError, metrics::Metrics};

pub async fn publish_block(
    message_broker: &Arc<dyn MessageBroker>,
    fuel_core: &Arc<dyn FuelCoreLike>,
    sealed_block: &Arc<SealedBlock>,
    telemetry: &Arc<Telemetry<Metrics>>,
) -> Result<(), PublishError> {
    let metadata = Metadata::new(fuel_core, sealed_block);
    let fuel_core = Arc::clone(fuel_core);
    let payload = MsgPayload::new(fuel_core, sealed_block, &metadata).await?;
    let encoded = payload.encode().await?;

    message_broker
        .publish_block(payload.message_id(), encoded.clone())
        .await?;

    if let Some(metrics) = telemetry.base_metrics() {
        metrics.update_publisher_success_metrics(
            &payload.subject(),
            encoded.len(),
        );
    }

    tracing::info!("New block submitted: {}", payload.block_height());
    Ok(())
}
