use std::sync::Arc;

use fuel_message_broker::MessageBroker;
use fuel_streams_core::{
    types::{Block, Transaction},
    FuelStreams,
};
use fuel_streams_domains::MsgPayload;
use fuel_streams_store::{
    db::Db,
    record::{DataEncoder, PacketBuilder, RecordEntity, RecordPacket},
};
use fuel_web_utils::shutdown::shutdown_broker_with_timeout;
use futures::{future::try_join_all, StreamExt};
use tokio::{sync::Semaphore, task::JoinSet};
use tokio_util::sync::CancellationToken;

use crate::{
    block_stats::BlockStats,
    errors::ConsumerError,
    ActionType,
    FuelStores,
};

#[derive(Debug)]
enum ProcessResult {
    Store(Result<BlockStats, ConsumerError>),
    Stream(Result<BlockStats, ConsumerError>),
}

pub async fn process_messages_from_broker(
    db: &Arc<Db>,
    token: &CancellationToken,
    message_broker: &Arc<dyn MessageBroker>,
    fuel_streams: &Arc<FuelStreams>,
    fuel_stores: &Arc<FuelStores>,
) -> Result<(), ConsumerError> {
    let semaphore = Arc::new(Semaphore::new(32));
    let mut join_set = JoinSet::new();

    while !token.is_cancelled() {
        let mut messages = message_broker.receive_blocks_stream(100).await?;
        while let Some(msg) = messages.next().await {
            let msg = msg?;
            let db = db.clone();
            let fuel_streams = fuel_streams.clone();
            let fuel_stores = fuel_stores.clone();
            let semaphore = semaphore.clone();
            let payload = msg.payload();
            let msg_payload = MsgPayload::decode(&payload).await?.arc();
            let packets = build_packets(&msg_payload);

            // Spawn store task
            let store_packets = packets.clone();
            let store_msg_payload = msg_payload.clone();
            join_set.spawn({
                let semaphore = semaphore.clone();
                async move {
                    let _permit = semaphore.acquire_owned().await?;
                    let result = handle_store_insertions(
                        &db,
                        &fuel_stores,
                        &store_packets,
                        &store_msg_payload,
                    )
                    .await;
                    Ok::<_, ConsumerError>(ProcessResult::Store(result))
                }
            });

            // Spawn stream task
            let stream_packets = packets.clone();
            let stream_msg_payload = msg_payload.clone();
            join_set.spawn({
                let semaphore = semaphore.clone();
                async move {
                    let _permit = semaphore.acquire_owned().await?;
                    let result = handle_stream_publishes(
                        &fuel_streams,
                        &stream_packets,
                        &stream_msg_payload,
                    )
                    .await;
                    Ok(ProcessResult::Stream(result))
                }
            });

            // TODO: msg is acking here before we wait the tasks to success
            // we should implement a background mechanism for recover failed msgs
            msg.ack().await.map_err(|e| {
                tracing::error!("Failed to ack message: {:?}", e);
                ConsumerError::MessageBrokerClient(e)
            })?;
        }

        // Process all results
        while let Some(result) = join_set.join_next().await {
            match result {
                Ok(Ok(ProcessResult::Store(store_result))) => {
                    let store_stats = store_result?;
                    match &store_stats.error {
                        Some(error) => store_stats.log_error(error),
                        None => store_stats.log_success(),
                    }
                }
                Ok(Ok(ProcessResult::Stream(stream_result))) => {
                    let stream_stats = stream_result?;
                    match &stream_stats.error {
                        Some(error) => stream_stats.log_error(error),
                        None => stream_stats.log_success(),
                    }
                }
                Ok(Err(e)) => tracing::error!("Task error: {}", e),
                Err(e) => tracing::error!("Task panicked: {}", e),
            }
        }
    }

    tracing::info!("Stopping actix server ...");
    shutdown_broker_with_timeout(message_broker).await;
    Ok(())
}

fn build_packets(msg_payload: &MsgPayload) -> Arc<Vec<RecordPacket>> {
    let block_packets = Block::build_packets(msg_payload);
    let tx_packets = Transaction::build_packets(msg_payload);
    let packets = block_packets
        .into_iter()
        .chain(tx_packets)
        .collect::<Vec<_>>();
    Arc::new(packets)
}

async fn handle_store_insertions(
    db: &Arc<Db>,
    fuel_stores: &Arc<FuelStores>,
    packets: &Arc<Vec<RecordPacket>>,
    msg_payload: &Arc<MsgPayload>,
) -> Result<BlockStats, ConsumerError> {
    let block_height = msg_payload.block_height();
    let stats = BlockStats::new(block_height.to_owned(), ActionType::Store);
    let mut db_tx = db.pool.begin().await?;
    let result = async {
        for packet in packets.iter() {
            let entity = packet.get_entity()?;
            match entity {
                RecordEntity::Block => {
                    fuel_stores
                        .blocks
                        .insert_record_with_transaction(&mut db_tx, packet)
                        .await?;
                }
                RecordEntity::Transaction => {
                    fuel_stores
                        .transactions
                        .insert_record_with_transaction(&mut db_tx, packet)
                        .await?;
                }
                RecordEntity::Input => {
                    fuel_stores
                        .inputs
                        .insert_record_with_transaction(&mut db_tx, packet)
                        .await?;
                }
                RecordEntity::Output => {
                    fuel_stores
                        .outputs
                        .insert_record_with_transaction(&mut db_tx, packet)
                        .await?;
                }
                RecordEntity::Receipt => {
                    fuel_stores
                        .receipts
                        .insert_record_with_transaction(&mut db_tx, packet)
                        .await?;
                }
                RecordEntity::Utxo => {
                    fuel_stores
                        .utxos
                        .insert_record_with_transaction(&mut db_tx, packet)
                        .await?;
                }
            }
        }
        Ok::<_, ConsumerError>(packets.len())
    }
    .await;

    match result {
        Ok(packet_count) => {
            db_tx.commit().await?;
            Ok(stats.finish(packet_count))
        }
        Err(e) => {
            db_tx.rollback().await?;
            Ok(stats.finish_with_error(e))
        }
    }
}

async fn handle_stream_publishes(
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
