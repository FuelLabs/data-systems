use crate::{
    processor::{
        Processor,
        StorageTypeConfig,
    },
    s3::S3TableName,
    DuneError,
};
use fuel_core_client::client::FuelClient;
use fuel_core_services::{
    stream::{
        BoxStream,
        IntoBoxStream,
    },
    RunnableService,
    RunnableTask,
    ServiceRunner,
    StateWatcher,
    TaskNextAction,
};
use fuel_core_types::{
    fuel_tx::AssetId,
    fuel_types::BlockHeight,
};
use fuel_indexer_types::events::BlockEvent;
use fuel_receipts_manager::{
    adapters::graphql_event_adapter::{
        create_graphql_event_adapter,
        GraphqlEventAdapterConfig,
        GraphqlFetcher,
    },
    port::FinalizedBlock,
};
use fuel_streams_domains::{
    blocks::Block,
    transactions::Transaction,
};
use futures::StreamExt;
use itertools::Itertools;
use std::{
    num::NonZeroUsize,
    sync::Arc,
};
use tokio::sync::watch;

pub struct Config {
    pub url: url::Url,
    pub starting_height: BlockHeight,
    pub storage_type: StorageTypeConfig,
    pub registry_blocks_request_batch_size: usize,
    pub registry_blocks_request_concurrency: usize,
}

pub struct UninitializedTask {
    config: Config,
    fetcher: GraphqlFetcher,
    shared: SharedState,
}

pub struct Task {
    height: watch::Sender<BlockHeight>,
    fetcher: GraphqlFetcher,
    blocks_stream: BoxStream<anyhow::Result<BlockEvent>>,
    post_interval: tokio::time::Interval,
    pending_events: Vec<BlockEvent>,
    processor: Processor,
    base_asset_id: AssetId,
}

#[derive(Clone)]
pub struct SharedState {
    block_height: watch::Sender<BlockHeight>,
}

impl SharedState {
    /// Awaits until the block height reaches at least `target_height`.
    /// Returns the reached block height.
    pub async fn await_block_height(
        &self,
        target_height: BlockHeight,
    ) -> anyhow::Result<BlockHeight> {
        let mut receiver = self.block_height.subscribe();
        while *receiver.borrow_and_update() < target_height {
            if receiver.changed().await.is_err() {
                return Err(anyhow::anyhow!(
                    "Block height channel closed before reaching target height"
                ));
            }
        }

        let height = *receiver.borrow();

        Ok(height)
    }
}

#[async_trait::async_trait]
impl RunnableService for UninitializedTask {
    const NAME: &'static str = "DuneService";
    type SharedData = SharedState;
    type Task = Task;
    type TaskParams = ();

    fn shared_data(&self) -> Self::SharedData {
        self.shared.clone()
    }

    async fn into_task(
        self,
        _: &StateWatcher,
        _: Self::TaskParams,
    ) -> anyhow::Result<Self::Task> {
        let Self {
            config,
            fetcher,
            shared,
        } = self;

        let client = FuelClient::new(&config.url)?;
        let base_asset_id = *client
            .chain_info()
            .await?
            .consensus_parameters
            .base_asset_id();

        let post_interval = tokio::time::interval(std::time::Duration::from_secs(1));

        let processor = Processor::new(config.storage_type).await?;

        let current_height = processor
            .load_latest_height()
            .await?
            .unwrap_or(config.starting_height);
        shared.block_height.send_replace(current_height);

        let mut task = Task {
            blocks_stream: futures::stream::pending().into_boxed(),
            height: shared.block_height,
            fetcher,
            post_interval,
            pending_events: vec![],
            processor,
            base_asset_id,
        };

        task.connect_block_stream().await?;

        Ok(task)
    }
}

impl RunnableTask for Task {
    async fn run(&mut self, watcher: &mut StateWatcher) -> TaskNextAction {
        tokio::select! {
            biased;

            _ = watcher.while_started() => {
                TaskNextAction::Stop
            }

            block = self.blocks_stream.next() => {
                match block {
                    Some(Ok(event)) => {
                        let current_height = if let Some(block) = self.pending_events.last() {
                            *block.header.height()
                        } else {
                            *self.height.borrow()
                        };

                        let Some(next_height) = current_height.succ() else {
                            tracing::error!("Block height overflowed when processing block event");
                            return TaskNextAction::Stop;
                        };

                        assert_eq!(next_height, *event.header.height());
                        self.pending_events.push(event);

                        TaskNextAction::Continue
                    }
                    Some(Err(e)) => {
                        tracing::error!("Error receiving block event: {e}; reconnecting stream");
                        match self.connect_block_stream().await {
                            Ok(_) => TaskNextAction::Continue,
                            Err(e) => {
                                tracing::error!("Failed to reconnect block stream: {e}");
                                TaskNextAction::Stop
                            }
                        }
                    }
                    None => {
                        tracing::warn!("Block event stream ended unexpectedly");
                        match self.connect_block_stream().await {
                            Ok(_) => TaskNextAction::Continue,
                            Err(e) => {
                                tracing::error!("Failed to reconnect block stream: {e}");
                                TaskNextAction::Stop
                            }
                        }
                    }
                }
            }

            _ = self.post_interval.tick() => {
                TaskNextAction::always_continue(self.post_blocks().await)
            }
        }
    }

    async fn shutdown(self) -> anyhow::Result<()> {
        Ok(())
    }
}

impl Task {
    async fn connect_block_stream(&mut self) -> anyhow::Result<()> {
        self.pending_events.clear();

        let height = *self.height.borrow();
        let next_height = height.succ().ok_or_else(|| {
            anyhow::anyhow!("Block height overflowed when connecting block stream")
        })?;
        let stream = self
            .fetcher
            .blocks_stream_starting_from(next_height)
            .await?
            .map(|result| {
                result.map(|block: FinalizedBlock| BlockEvent {
                    header: block.header,
                    consensus: block.consensus,
                    transactions: block.transactions,
                    statuses: block.statuses,
                })
            })
            .into_boxed();
        self.blocks_stream = stream;

        Ok(())
    }

    async fn post_blocks(&mut self) -> anyhow::Result<()> {
        if self.pending_events.is_empty() {
            return Ok(())
        }

        let last_height = *self
            .pending_events
            .last()
            .expect("We have pending events; qed")
            .header
            .height();

        let blocks_and_txs: Vec<_> = self
            .pending_events
            .iter()
            .map(|event| {
                let block = Block::new(
                    &event.header,
                    event.consensus.clone(),
                    event.transactions.len(),
                )?;
                let transaction = event
                    .transactions
                    .iter()
                    .zip(event.statuses.iter())
                    .map(|(tx, status)| {
                        Transaction::new(
                            &status.id.into(),
                            tx.as_ref(),
                            &status.into(),
                            &self.base_asset_id,
                            status.result.receipts(),
                        )
                    })
                    .collect();

                Ok::<_, anyhow::Error>((block, transaction))
            })
            .try_collect()?;

        process_batch(&self.processor, &blocks_and_txs)
            .await
            .map_err(|err| {
                anyhow::anyhow!(
                    "Failed to process batch of blocks and transactions: {err}"
                )
            })?;

        self.processor.save_latest_height(last_height).await?;
        self.height.send_replace(last_height);

        self.pending_events.clear();

        Ok(())
    }
}

pub fn new_service(config: Config) -> anyhow::Result<ServiceRunner<UninitializedTask>> {
    let graphql_config = GraphqlEventAdapterConfig {
        client: Arc::new(FuelClient::new(&config.url)?),
        heartbeat_capacity: NonZeroUsize::new(100_000).expect("Is not zero; qed"),
        event_capacity: NonZeroUsize::new(100_000).expect("Is not zero; qed"),
        blocks_request_batch_size: config.registry_blocks_request_batch_size,
        blocks_request_concurrency: config.registry_blocks_request_concurrency,
    };
    let fetcher = create_graphql_event_adapter(graphql_config);
    let (height, _) = watch::channel(config.starting_height);
    let task = UninitializedTask {
        config,
        fetcher,
        shared: SharedState {
            block_height: height,
        },
    };

    Ok(ServiceRunner::new(task))
}

pub async fn process_batch(
    processor: &Processor,
    blocks_and_txs: &[(Block, Vec<Transaction>)],
) -> anyhow::Result<()> {
    let blocks_task = async {
        let batches = processor.calculate_blocks_batches(blocks_and_txs)?;
        processor
            .process_range(batches, S3TableName::Blocks)
            .await?;
        Ok::<_, DuneError>(())
    };

    let tx_task = async {
        let batches = processor.calculate_txs_batches(blocks_and_txs)?;
        processor
            .process_range(batches, S3TableName::Transactions)
            .await?;
        Ok::<_, DuneError>(())
    };

    let receipts_task = async {
        let batches = processor.calculate_receipts_batches(blocks_and_txs)?;
        processor
            .process_range(batches, S3TableName::Receipts)
            .await?;
        Ok::<_, DuneError>(())
    };

    tokio::try_join!(blocks_task, tx_task, receipts_task)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::processor::StorageTypeConfig;
    use fuel_core::service::{
        Config,
        FuelService,
    };
    use fuel_core_client::client::FuelClient;
    use fuel_core_services::Service;
    use std::time::Duration;
    use url::Url;

    #[tokio::test]
    async fn service_progress_height() {
        let node = FuelService::new_node(Config::local_node()).await.unwrap();

        let config = super::Config {
            url: Url::parse(format!("http://{}", node.bound_address).as_str()).unwrap(),
            starting_height: 0u32.into(),
            storage_type: StorageTypeConfig::S3,
            registry_blocks_request_batch_size: 10,
            registry_blocks_request_concurrency: 100,
        };

        // Given
        let service = super::new_service(config).unwrap();
        service.start_and_await().await.unwrap();

        // When
        let client = FuelClient::from(node.bound_address);
        client.produce_blocks(100, None).await.unwrap();

        // Then
        let shared = service.shared.clone();
        let result = tokio::time::timeout(
            Duration::from_secs(10),
            shared.await_block_height(100u32.into()),
        )
        .await;
        assert!(
            result.is_ok(),
            "Timed out waiting for block height to reach 100"
        );
        let await_result = result.unwrap();
        let height = await_result.expect("Awaiting block height to reach 100");
        assert!(height >= 100u32.into());
    }
}
