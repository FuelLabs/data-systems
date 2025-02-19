use std::sync::Arc;

use fuel_message_broker::{Message, NatsMessageBroker, NatsQueue};
use fuel_streams_core::{
    types::{Block, Transaction},
    FuelStreams,
};
use fuel_streams_domains::MsgPayload;
use fuel_streams_store::{
    db::Db,
    record::{DataEncoder, PacketBuilder, RecordPacket},
};
use fuel_web_utils::{
    shutdown::shutdown_broker_with_timeout,
    telemetry::Telemetry,
};
use futures::{future::try_join_all, StreamExt};
use tokio::{
    sync::Semaphore,
    task::{JoinError, JoinSet},
};
use tokio_util::sync::CancellationToken;

use super::{
    block_stats::{ActionType, BlockStats},
    retry::RetryService,
};
use crate::{errors::ConsumerError, metrics::Metrics, FuelStores};

const MAX_CONCURRENT_TASKS: usize = 32;
const BATCH_SIZE: usize = 100;

#[derive(Debug)]
enum ProcessResult {
    Store(Result<BlockStats, ConsumerError>),
    Stream(Result<BlockStats, ConsumerError>),
}

pub struct BlockExecutor {
    db: Arc<Db>,
    message_broker: Arc<NatsMessageBroker>,
    fuel_streams: Arc<FuelStreams>,
    fuel_stores: Arc<FuelStores>,
    semaphore: Arc<Semaphore>,
    max_tasks: usize,
    telemetry: Arc<Telemetry<Metrics>>,
}

impl BlockExecutor {
    pub fn new(
        db: Arc<Db>,
        message_broker: &Arc<NatsMessageBroker>,
        fuel_streams: &Arc<FuelStreams>,
        telemetry: Arc<Telemetry<Metrics>>,
    ) -> Self {
        let pool_size = db.pool.size() as usize;
        let max_tasks = pool_size.saturating_sub(5).min(MAX_CONCURRENT_TASKS);
        let semaphore = Arc::new(Semaphore::new(max_tasks));
        let fuel_stores = FuelStores::new(&db).arc();
        Self {
            db,
            semaphore,
            message_broker: message_broker.clone(),
            fuel_streams: fuel_streams.clone(),
            fuel_stores: fuel_stores.clone(),
            max_tasks,
            telemetry,
        }
    }

    pub async fn start(
        &self,
        token: &CancellationToken,
    ) -> Result<(), ConsumerError> {
        let mut join_set = JoinSet::new();
        tracing::info!(
            "Starting consumer with max concurrent tasks: {}",
            self.max_tasks
        );
        let telemetry = self.telemetry.clone();
        let queue = NatsQueue::BlockImporter(self.message_broker.clone());
        while !token.is_cancelled() {
            tokio::select! {
                msg_result = queue.subscribe(BATCH_SIZE) => {
                    let mut messages = msg_result?;
                    while let Some(msg) = messages.next().await {
                        let msg = msg?;
                        self.spawn_processing_tasks(
                            msg,
                            &mut join_set,
                        )
                        .await?;
                    }
                }
                Some(result) = join_set.join_next() => {
                    Self::handle_task_result(result, &telemetry).await?;
                }
            }
        }

        // Wait for all tasks to finish
        while let Some(result) = join_set.join_next().await {
            Self::handle_task_result(result, &telemetry).await?;
        }

        tracing::info!("Stopping broker ...");
        shutdown_broker_with_timeout(&self.message_broker).await;
        tracing::info!("Broker stopped successfully!");
        Ok(())
    }

    async fn spawn_processing_tasks(
        &self,
        msg: Box<dyn Message>,
        join_set: &mut JoinSet<Result<ProcessResult, ConsumerError>>,
    ) -> Result<(), ConsumerError> {
        let db = self.db.clone();
        let semaphore = self.semaphore.clone();
        let fuel_streams = self.fuel_streams.clone();
        let fuel_stores = self.fuel_stores.clone();
        let payload = msg.payload();
        let msg_payload = MsgPayload::decode(&payload).await?.arc();
        let packets = Self::build_packets(&msg_payload);
        join_set.spawn({
            let semaphore = semaphore.clone();
            let packets = packets.clone();
            let msg_payload = msg_payload.clone();
            let db = db.clone();
            let fuel_stores = fuel_stores.clone();
            async move {
                let _permit = semaphore.acquire().await?;
                let result =
                    handle_stores(&db, &fuel_stores, &packets, &msg_payload)
                        .await;
                Ok::<_, ConsumerError>(ProcessResult::Store(result))
            }
        });

        join_set.spawn({
            let semaphore = semaphore.clone();
            let packets = packets.clone();
            let msg_payload = msg_payload.clone();
            let fuel_streams = fuel_streams.clone();
            async move {
                let _permit = semaphore.acquire_owned().await?;
                let result =
                    handle_streams(&fuel_streams, &packets, &msg_payload).await;
                Ok(ProcessResult::Stream(result))
            }
        });

        msg.ack().await.map_err(|e| {
            tracing::error!("Failed to ack message: {:?}", e);
            ConsumerError::MessageBrokerClient(e)
        })?;

        Ok(())
    }

    async fn handle_task_result(
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
}

async fn handle_stores(
    db: &Arc<Db>,
    fuel_stores: &Arc<FuelStores>,
    packets: &Arc<Vec<RecordPacket>>,
    msg_payload: &Arc<MsgPayload>,
) -> Result<BlockStats, ConsumerError> {
    let block_height = msg_payload.block_height();
    let stats = BlockStats::new(block_height.to_owned(), ActionType::Store);
    let retry_service = RetryService::default();
    let result = retry_service
        .with_retry("store_insertions", || async {
            let mut tx = db.pool.begin().await?;
            for packet in packets.iter() {
                fuel_stores.insert_by_entity(&mut tx, packet).await?;
            }
            tx.commit().await?;
            Ok(packets.len())
        })
        .await;

    match result {
        Ok(packet_count) => Ok(stats.finish(packet_count)),
        Err(e) => Ok(stats.finish_with_error(e)),
    }
}

async fn handle_streams(
    fuel_streams: &Arc<FuelStreams>,
    packets: &Arc<Vec<RecordPacket>>,
    msg_payload: &Arc<MsgPayload>,
) -> Result<BlockStats, ConsumerError> {
    let block_height = msg_payload.block_height();
    let stats = BlockStats::new(block_height.to_owned(), ActionType::Stream);
    let publish_futures = packets.iter().map(|packet| {
        let packet = packet.to_owned();
        fuel_streams.publish_by_entity(packet.arc())
    });

    match try_join_all(publish_futures).await {
        Ok(_) => Ok(stats.finish(packets.len())),
        Err(e) => Ok(stats.finish_with_error(ConsumerError::from(e))),
    }
}
