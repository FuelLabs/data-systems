use std::sync::Arc;

use fuel_streams_core::FuelStreams;
use fuel_streams_domains::MsgPayload;
use fuel_streams_store::record::{RecordEntity, RecordPacket};
use futures::future::try_join_all;

use super::block_stats::{ActionType, BlockStats};
use crate::errors::ConsumerError;

pub async fn handle_stream_publishes(
    fuel_streams: &Arc<FuelStreams>,
    packets: &Arc<Vec<RecordPacket>>,
    msg_payload: &Arc<MsgPayload>,
) -> Result<BlockStats, ConsumerError> {
    let block_height = msg_payload.block_height();
    let stats = BlockStats::new(block_height.to_owned(), ActionType::Stream);
    let publish_futures = packets.iter().map(|packet| async {
        let entity = packet.get_entity()?;
        let subject = packet.subject_str();
        let payload = packet.to_owned().value.into();
        match entity {
            RecordEntity::Block => {
                fuel_streams.blocks.publish(&subject, payload).await
            }
            RecordEntity::Transaction => {
                fuel_streams.transactions.publish(&subject, payload).await
            }
            RecordEntity::Input => {
                fuel_streams.inputs.publish(&subject, payload).await
            }
            RecordEntity::Output => {
                fuel_streams.outputs.publish(&subject, payload).await
            }
            RecordEntity::Receipt => {
                fuel_streams.receipts.publish(&subject, payload).await
            }
            RecordEntity::Utxo => {
                fuel_streams.utxos.publish(&subject, payload).await
            }
        }
    });

    match try_join_all(publish_futures).await {
        Ok(_) => Ok(stats.finish(packets.len())),
        Err(e) => Ok(stats.finish_with_error(ConsumerError::from(e))),
    }
}
