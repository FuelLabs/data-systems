use std::sync::Arc;

use fuel_data_parser::DataEncoder;
use fuel_message_broker::{
    Message as NatsMessage,
    NatsMessageBroker,
    NatsQueue,
};
use fuel_streams_core::{
    types::{BlockTimestamp, Message},
    FuelStreams,
};
use fuel_streams_domains::{
    infra::{
        db::Db,
        record::{PacketBuilder, RecordEntity, RecordPacket},
        repository::Repository,
    },
    messages::{Message as MessageEntity, MessageDbItem},
    MsgPayload,
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
use crate::{errors::ConsumerError, metrics::Metrics};

const MAX_CONCURRENT_TASKS: usize = 32;
const BATCH_SIZE: usize = 100;

#[derive(Debug)]
enum ProcessResult {
    Store(Result<BlockStats, ConsumerError>),
    Stream(Result<BlockStats, ConsumerError>),
}

pub struct BlockEventExecutor {
    db: Arc<Db>,
    message_broker: Arc<NatsMessageBroker>,
    fuel_streams: Arc<FuelStreams>,
    semaphore: Arc<Semaphore>,
    telemetry: Arc<Telemetry<Metrics>>,
}

impl BlockEventExecutor {
    pub fn new(
        db: Arc<Db>,
        message_broker: &Arc<NatsMessageBroker>,
        fuel_streams: &Arc<FuelStreams>,
        telemetry: Arc<Telemetry<Metrics>>,
    ) -> Self {
        let semaphore = Arc::new(Semaphore::new(MAX_CONCURRENT_TASKS));
        Self {
            db,
            semaphore,
            message_broker: message_broker.clone(),
            fuel_streams: fuel_streams.clone(),
            telemetry,
        }
    }

    pub async fn start(
        &self,
        token: &CancellationToken,
    ) -> Result<(), ConsumerError> {
        let mut join_set = JoinSet::new();
        tracing::info!(
            "Starting event consumer with max concurrent tasks: {}",
            MAX_CONCURRENT_TASKS
        );

        let telemetry = self.telemetry.clone();
        let queue = NatsQueue::BlockEvent(self.message_broker.clone());

        while !token.is_cancelled() {
            tokio::select! {
                msg_result = queue.subscribe(BATCH_SIZE) => {
                    let mut messages = msg_result?;
                    while let Some(msg) = messages.next().await {
                        let msg= msg?;
                        self.spawn_processing_tasks(msg, &mut join_set,)
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
        msg: Box<dyn NatsMessage>,
        join_set: &mut JoinSet<Result<ProcessResult, ConsumerError>>,
    ) -> Result<(), ConsumerError> {
        let db = self.db.clone();
        let semaphore = self.semaphore.clone();
        let fuel_streams = self.fuel_streams.clone();
        let payload = msg.payload();
        let msg_payload = MsgPayload::decode_json(&payload)?.arc();
        let message_packets = Self::extract_message_packets(&msg_payload);

        join_set.spawn({
            let semaphore = semaphore.clone();
            let message_packets = message_packets.clone();
            let msg_payload = msg_payload.clone();
            async move {
                let _permit = semaphore.acquire().await?;
                let result =
                    handle_store_messages(&db, &message_packets, &msg_payload)
                        .await;
                Ok::<_, ConsumerError>(ProcessResult::Store(result))
            }
        });

        join_set.spawn({
            let semaphore = semaphore.clone();
            let message_packets = message_packets.clone();
            let msg_payload = msg_payload.clone();
            let fuel_streams = fuel_streams.clone();
            async move {
                let _permit = semaphore.acquire_owned().await?;
                let result = handle_stream_messages(
                    &fuel_streams,
                    &message_packets,
                    &msg_payload,
                )
                .await;
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
                    None => store_stats.log_success("[ON_BLOCK_EVENT]"),
                }
            }
            Ok(Ok(ProcessResult::Stream(stream_result))) => {
                let stream_stats = stream_result?;
                match &stream_stats.error {
                    Some(error) => stream_stats.log_error(error),
                    None => stream_stats.log_success("[ON_BLOCK_EVENT]"),
                }
            }
            Ok(Err(e)) => tracing::error!("Task error: {}", e),
            Err(e) => tracing::error!("Task panicked: {}", e),
        }
        Ok(())
    }

    fn extract_message_packets(
        msg_payload: &MsgPayload,
    ) -> Arc<Vec<RecordPacket>> {
        // Use the Message type's build_packets method to extract only message packets
        let message_packets = Message::build_packets(msg_payload);
        Arc::new(message_packets)
    }
}

async fn handle_store_messages(
    db: &Arc<Db>,
    packets: &Arc<Vec<RecordPacket>>,
    msg_payload: &Arc<MsgPayload>,
) -> Result<BlockStats, ConsumerError> {
    let block_height = msg_payload.block_height();
    let stats = BlockStats::new(block_height.to_owned(), ActionType::Store);
    let retry_service = RetryService::default();
    let result = retry_service
        .with_retry("store_messages", || async {
            let mut tx = db.pool.begin().await?;
            for packet in packets.iter() {
                let subject_id = packet.subject_id();
                let entity = RecordEntity::from_subject_id(&subject_id)?;
                if matches!(entity, RecordEntity::Message) {
                    let db_item = MessageDbItem::try_from(packet)?;
                    MessageEntity::insert_with_transaction(&mut tx, &db_item)
                        .await?;
                }
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

async fn handle_stream_messages(
    fuel_streams: &Arc<FuelStreams>,
    packets: &Arc<Vec<RecordPacket>>,
    msg_payload: &Arc<MsgPayload>,
) -> Result<BlockStats, ConsumerError> {
    let block_height = msg_payload.block_height();
    let stats = BlockStats::new(block_height.to_owned(), ActionType::Stream);
    let now = BlockTimestamp::now();

    let message_packets = packets.iter().filter(|packet| {
        let subject_id = packet.subject_id();
        if let Ok(entity) = RecordEntity::from_subject_id(&subject_id) {
            matches!(entity, RecordEntity::Message)
        } else {
            false
        }
    });

    let publish_futures = message_packets.map(|packet| {
        let packet = packet.to_owned();
        let packet = packet.with_start_time(now);
        fuel_streams.publish_by_entity(packet.arc())
    });

    match try_join_all(publish_futures).await {
        Ok(_) => Ok(stats.finish(packets.len())),
        Err(e) => Ok(stats.finish_with_error(ConsumerError::from(e))),
    }
}
