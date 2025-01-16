use std::sync::Arc;

use fuel_message_broker::{Message, MessageBroker};
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
use futures::{stream::FuturesUnordered, StreamExt};
use tokio::sync::Semaphore;
use tokio_util::sync::CancellationToken;

use crate::{block_stats::BlockStats, errors::ConsumerError, FuelStores};

pub async fn process_messages_from_broker(
    db: &Arc<Db>,
    token: &CancellationToken,
    message_broker: &Arc<dyn MessageBroker>,
    fuel_streams: &Arc<FuelStreams>,
    fuel_stores: &Arc<FuelStores>,
) -> Result<(), ConsumerError> {
    let semaphore = Arc::new(Semaphore::new(32));
    while !token.is_cancelled() {
        let mut messages = message_broker.receive_blocks_stream(100).await?;
        let mut futs = FuturesUnordered::new();

        // Process block on each new message
        while let Some(msg) = messages.next().await {
            let msg = msg?;
            let db = db.clone();
            let fuel_streams = fuel_streams.clone();
            let fuel_stores = fuel_stores.clone();
            let permit = semaphore.clone().acquire_owned().await?;
            futs.push(tokio::spawn(async move {
                let result =
                    process_block(&db, msg, &fuel_streams, &fuel_stores)
                        .await?;
                drop(permit);
                Ok::<_, ConsumerError>(result)
            }));
        }

        // Execute the thread and log results for each block from BlockStats
        while let Some(result) = futs.next().await {
            let block_stats = &result??;
            match &block_stats.error {
                Some(error) => block_stats.log_error(error),
                None => block_stats.log_success(),
            }
        }
    }

    tracing::info!("Stopping actix server ...");
    shutdown_broker_with_timeout(message_broker).await;
    Ok(())
}

async fn process_block(
    db: &Arc<Db>,
    msg: Box<dyn Message>,
    fuel_streams: &Arc<FuelStreams>,
    fuel_stores: &Arc<FuelStores>,
) -> Result<BlockStats, ConsumerError> {
    let payload = msg.payload();
    let msg_payload = MsgPayload::decode(&payload).await?;
    let height = msg_payload.block_height().to_owned();
    let stats = BlockStats::new(height);
    let block_packets = Block::build_packets(&msg_payload);
    let tx_packets = Transaction::build_packets(&msg_payload);
    let packets = block_packets
        .into_iter()
        .chain(tx_packets)
        .collect::<Vec<_>>();

    let result = process_packets(db, &packets, fuel_streams, fuel_stores).await;
    msg.ack().await.map_err(|e| {
        tracing::error!("Failed to ack message: {:?}", e);
        ConsumerError::MessageBrokerClient(e)
    })?;

    match result {
        Ok(packet_count) => Ok(stats.finish(packet_count)),
        Err(e) => Ok(stats.finish_with_error(e)),
    }
}

async fn process_packets(
    db: &Arc<Db>,
    packets: &[RecordPacket],
    fuel_streams: &Arc<FuelStreams>,
    fuel_stores: &Arc<FuelStores>,
) -> Result<usize, ConsumerError> {
    let mut db_tx = db.pool.begin().await?;
    for packet in packets {
        let entity = packet.get_entity()?;
        match entity {
            RecordEntity::Block => {
                let record = fuel_stores
                    .blocks
                    .insert_record_with_transaction(&mut db_tx, packet)
                    .await?;
                fuel_streams.blocks.publish(&record).await?;
            }
            RecordEntity::Transaction => {
                let record = fuel_stores
                    .transactions
                    .insert_record_with_transaction(&mut db_tx, packet)
                    .await?;
                fuel_streams.transactions.publish(&record).await?;
            }
            RecordEntity::Input => {
                let record = fuel_stores
                    .inputs
                    .insert_record_with_transaction(&mut db_tx, packet)
                    .await?;
                fuel_streams.inputs.publish(&record).await?;
            }
            RecordEntity::Output => {
                let record = fuel_stores
                    .outputs
                    .insert_record_with_transaction(&mut db_tx, packet)
                    .await?;
                fuel_streams.outputs.publish(&record).await?;
            }
            RecordEntity::Receipt => {
                let record = fuel_stores
                    .receipts
                    .insert_record_with_transaction(&mut db_tx, packet)
                    .await?;
                fuel_streams.receipts.publish(&record).await?;
            }
            RecordEntity::Utxo => {
                let record = fuel_stores
                    .utxos
                    .insert_record_with_transaction(&mut db_tx, packet)
                    .await?;
                fuel_streams.utxos.publish(&record).await?;
            }
        }
    }

    db_tx.commit().await?;
    Ok(packets.len())
}
