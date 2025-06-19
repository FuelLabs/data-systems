use std::sync::Arc;

use fuel_data_parser::DataEncoder;
use fuel_message_broker::{
    Message as NatsMessage,
    NatsMessageBroker,
    NatsQueue,
};
use fuel_streams_core::{
    types::{
        Block,
        BlockHeight,
        BlockTimestamp,
        Input,
        Message,
        Output,
        Receipt,
        Transaction,
        Utxo,
    },
    FuelStreams,
};
use fuel_streams_domains::{
    blocks::{BlockDbItem, BlocksQuery},
    infra::{
        db::{Db, DbTransaction},
        record::{PacketBuilder, RecordEntity, RecordPacket},
        repository::Repository,
        RepositoryError,
    },
    inputs::InputDbItem,
    messages::MessageDbItem,
    outputs::OutputDbItem,
    predicates::{Predicate, PredicateDbItem},
    receipts::ReceiptDbItem,
    transactions::TransactionDbItem,
    utxos::UtxoDbItem,
    MsgPayload,
};
use fuel_web_utils::{
    shutdown::shutdown_broker_with_timeout,
    telemetry::Telemetry,
};
use futures::{future::try_join_all, StreamExt};
use tokio::{sync::Semaphore, task::JoinError};
use tokio_util::sync::CancellationToken;

use super::{
    block_stats::{ActionType, BlockStats},
    retry::RetryService,
};
use crate::{errors::ConsumerError, metrics::Metrics};

#[derive(Debug)]
enum ProcessResult {
    Store(Result<BlockStats, ConsumerError>),
}

pub struct BlockExecutor {
    db: Arc<Db>,
    message_broker: Arc<NatsMessageBroker>,
    fuel_streams: Arc<FuelStreams>,
    semaphore: Arc<Semaphore>,
    telemetry: Arc<Telemetry<Metrics>>,
    concurrent_tasks: usize,
}

impl BlockExecutor {
    pub fn new(
        db: Arc<Db>,
        message_broker: &Arc<NatsMessageBroker>,
        fuel_streams: &Arc<FuelStreams>,
        telemetry: Arc<Telemetry<Metrics>>,
        concurrent_tasks: usize,
    ) -> Self {
        let semaphore = Arc::new(Semaphore::new(concurrent_tasks));
        Self {
            db,
            semaphore,
            message_broker: message_broker.clone(),
            fuel_streams: fuel_streams.clone(),
            telemetry,
            concurrent_tasks,
        }
    }

    pub async fn start(
        &self,
        token: &CancellationToken,
    ) -> Result<(), ConsumerError> {
        tracing::info!(
            "Starting consumer with max concurrent tasks: {}",
            self.concurrent_tasks
        );
        let queue = NatsQueue::BlockImporter(self.message_broker.clone());

        while !token.is_cancelled() {
            let mut messages = queue.subscribe(self.concurrent_tasks).await?;
            while let Some(msg) = messages.next().await {
                let msg = msg?;
                let semaphore = self.semaphore.clone();
                let permit = match semaphore.clone().acquire_owned().await {
                    Ok(p) => p,
                    Err(_) => continue,
                };
                self.spawn_processing_tasks(msg, permit).await?;
            }
        }

        tracing::info!("Stopping broker ...");
        shutdown_broker_with_timeout(&self.message_broker).await;
        tracing::info!("Broker stopped successfully!");
        Ok(())
    }

    async fn spawn_processing_tasks(
        &self,
        msg: Box<dyn NatsMessage>,
        permit: tokio::sync::OwnedSemaphorePermit,
    ) -> Result<(), ConsumerError> {
        let db = self.db.clone();
        let fuel_streams = self.fuel_streams.clone();
        let payload = msg.payload();
        let msg_payload = MsgPayload::decode_json(&payload)?.arc();
        let packets = Self::build_packets(&msg_payload);
        let telemetry = self.telemetry.clone();

        tokio::spawn({
            let packets: Arc<Vec<RecordPacket>> = packets.clone();
            let msg_payload = msg_payload.clone();
            let telemetry = telemetry.clone();
            async move {
                let query = BlocksQuery {
                    height: Some(msg_payload.block_height()),
                    ..Default::default()
                };
                let block = Block::find_one(db.pool_ref(), &query).await;
                if block.is_ok() {
                    tracing::info!(
                        "[#{}] Block already processed",
                        msg_payload.block_height()
                    );
                    let _ = msg.ack().await.map_err(|e| {
                        tracing::error!("Failed to ack message: {:?}", e);
                        ConsumerError::MessageBrokerClient(e)
                    });
                    tracing::info!(
                        "[#{}] Message acknowledged",
                        msg_payload.block_height()
                    );
                    drop(permit);
                    return;
                }
                let _ =
                    handle_streams_task(&fuel_streams, &packets, &msg_payload)
                        .await;
                let result = handle_stores(&db, &packets, &msg_payload).await;
                // Drop semaphore as soon as store is completed
                drop(permit);
                let result = match result {
                    Ok(stats) => {
                        if stats.error.is_none() {
                            tokio::spawn(async move {
                                let _ = msg.ack().await.map_err(|e| {
                                    tracing::error!(
                                        "Failed to ack message: {:?}",
                                        e
                                    );
                                    ConsumerError::MessageBrokerClient(e)
                                });
                                tracing::info!(
                                    "[#{}] Message acknowledged",
                                    stats.block_height
                                );
                            });
                        }
                        Ok(stats)
                    }
                    Err(e) => Err(e),
                };
                let _ = Self::handle_task_result(
                    Ok(Ok::<_, ConsumerError>(ProcessResult::Store(result))),
                    &telemetry,
                );
            }
        });

        Ok(())
    }

    fn handle_task_result(
        result: Result<Result<ProcessResult, ConsumerError>, JoinError>,
        telemetry: &Arc<Telemetry<Metrics>>,
    ) -> Result<(), ConsumerError> {
        match result {
            Ok(Ok(ProcessResult::Store(store_result))) => {
                let store_stats = store_result?;
                if let Some(metrics) = telemetry.base_metrics() {
                    metrics.update_from_stats(&store_stats)
                }

                match &store_stats.error {
                    Some(error) => store_stats.log_error(error),
                    None => store_stats.log_success(),
                }
            }
            Ok(Err(e)) => tracing::error!("Task error: {}", e),
            Err(e) => tracing::error!("Task panicked: {}", e),
        }
        Ok(())
    }

    fn build_packets(msg_payload: &MsgPayload) -> Arc<Vec<RecordPacket>> {
        let block_packets = Block::build_packets(msg_payload);
        let message_packets = Message::build_packets(msg_payload);
        let tx_packets = Transaction::build_packets(msg_payload);
        let packets = block_packets
            .into_iter()
            .chain(tx_packets)
            .chain(message_packets)
            .collect::<Vec<_>>();
        Arc::new(packets)
    }
}

async fn handle_insertions(
    tx: &mut DbTransaction,
    packets: &Arc<Vec<RecordPacket>>,
) -> Result<(), ConsumerError> {
    // First insert blocks
    for packet in packets.iter() {
        let subject_id = packet.subject_id();
        let entity = RecordEntity::from_subject_id(&subject_id)?;
        match entity {
            RecordEntity::Block => {
                let db_item = BlockDbItem::try_from(packet)?;
                Block::insert_with_transaction(tx, &db_item).await?;
            }
            RecordEntity::Message => {
                let db_item = MessageDbItem::try_from(packet)?;
                Message::insert_with_transaction(tx, &db_item).await?;
            }
            RecordEntity::Transaction => {
                let db_item = TransactionDbItem::try_from(packet)?;
                Transaction::insert_with_transaction(tx, &db_item).await?;
            }
            RecordEntity::Input => {
                let db_item = InputDbItem::try_from(packet)?;
                Input::insert_with_transaction(tx, &db_item).await?;
            }
            RecordEntity::Output => {
                let db_item = OutputDbItem::try_from(packet)?;
                Output::insert_with_transaction(tx, &db_item).await?;
            }
            RecordEntity::Receipt => {
                let db_item = ReceiptDbItem::try_from(packet)?;
                Receipt::insert_with_transaction(tx, &db_item).await?;
            }
            _ => {}
        }
    }
    Ok(())
}

async fn handle_stores(
    db: &Arc<Db>,
    packets: &Arc<Vec<RecordPacket>>,
    msg_payload: &Arc<MsgPayload>,
) -> Result<BlockStats, ConsumerError> {
    let block_height = msg_payload.block_height();
    let stats = BlockStats::new(block_height.to_owned(), ActionType::Store);
    let retry_service = RetryService::default();
    let result = retry_service
        .with_retry("store_insertions", || async {
            let mut tx = db.pool.begin().await?;
            // Insert blocks, messages, transactions, inputs, outputs, and receipts
            match handle_insertions(&mut tx, packets).await {
                Ok(_) => {
                    let block_propagation_ms =
                        stats.calculate_block_propagation_ms();
                    update_block_propagation_ms(
                        &mut tx,
                        block_height,
                        block_propagation_ms,
                    )
                    .await?;
                    tx.commit().await?;
                    // Then, insert separately predicates and UTXOs
                    for packet in packets.iter() {
                        let subject_id = packet.subject_id();
                        let entity =
                            RecordEntity::from_subject_id(&subject_id)?;

                        match entity {
                            RecordEntity::Predicate => {
                                let mut db_item =
                                    PredicateDbItem::try_from(packet)?;
                                Predicate::upsert_as_relation(db, &mut db_item)
                                    .await?;
                            }
                            RecordEntity::Utxo => {
                                let db_item = UtxoDbItem::try_from(packet)?;
                                Utxo::insert(db.pool_ref(), &db_item).await?;
                            }
                            _ => {}
                        }
                    }
                    Ok(packets.len())
                }
                Err(e) => {
                    tracing::error!(
                        "[#{}] Failed to insert packets: {:?}",
                        block_height,
                        e
                    );
                    tx.rollback().await?;
                    Err(e)
                }
            }
        })
        .await;

    match result {
        Ok(packet_count) => Ok(stats.finish(packet_count)),
        Err(e) => {
            if let ConsumerError::Repository(RepositoryError::Sqlx(
                sqlx::Error::Database(db_error),
            )) = &e
            {
                if db_error.is_unique_violation() {
                    tracing::info!(
                        "[#{}] Ignoring unique constraint violation - block already processed",
                        block_height
                    );
                    return Ok(stats.finish(packets.len()));
                }
            }
            Ok(stats.finish_with_error(e))
        }
    }
}

fn handle_streams_task(
    fuel_streams: &Arc<FuelStreams>,
    packets: &Arc<Vec<RecordPacket>>,
    msg_payload: &Arc<MsgPayload>,
) -> tokio::task::JoinHandle<()> {
    let packets = packets.clone();
    let msg_payload = msg_payload.clone();
    let fuel_streams = fuel_streams.clone();
    tokio::spawn(async move {
        let result_streams =
            handle_streams(&fuel_streams, &packets, &msg_payload).await;
        if let Ok(stream_stats) = result_streams {
            match &stream_stats.error {
                Some(error) => stream_stats.log_error(error),
                None => stream_stats.log_success(),
            }
        }
    })
}

async fn handle_streams(
    fuel_streams: &Arc<FuelStreams>,
    packets: &Arc<Vec<RecordPacket>>,
    msg_payload: &Arc<MsgPayload>,
) -> Result<BlockStats, ConsumerError> {
    tracing::info!("[#{}] Streaming packets", msg_payload.block_height());
    let block_height = msg_payload.block_height();
    let stats = BlockStats::new(block_height.to_owned(), ActionType::Stream);
    let now = BlockTimestamp::now();

    let publish_futures = packets.iter().map(|packet| {
        let packet = packet.to_owned();
        let packet = packet.with_start_time(now);
        fuel_streams.publish_by_entity(packet.arc())
    });

    match try_join_all(publish_futures).await {
        Ok(_) => Ok(stats.finish(packets.len())),
        Err(e) => Ok(stats.finish_with_error(ConsumerError::from(e))),
    }
}

pub async fn update_block_propagation_ms(
    tx: &mut DbTransaction,
    block_height: BlockHeight,
    propagation_ms: u64,
) -> Result<(), ConsumerError> {
    sqlx::query(
        "UPDATE blocks SET block_propagation_ms = $1 WHERE block_height = $2",
    )
    .bind(propagation_ms as i64)
    .bind(block_height)
    .execute(&mut **tx)
    .await?;
    Ok(())
}
