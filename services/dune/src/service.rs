use crate::{
    DuneError,
    block_buffer::{
        DiskBuffer,
        FinalizedBatchFiles,
    },
    processor::{
        Processor,
        StorageTypeConfig,
    },
    s3::S3TableName,
};
use fuel_core_client::client::FuelClient;
use fuel_core_services::{
    RunnableService,
    RunnableTask,
    ServiceRunner,
    StateWatcher,
    TaskNextAction,
    stream::{
        BoxStream,
        IntoBoxStream,
    },
};
use fuel_core_types::{
    fuel_tx::AssetId,
    fuel_types::BlockHeight,
};
use fuel_indexer_types::events::BlockEvent;
use fuel_receipts_manager::{
    adapters::graphql_event_adapter::{
        GraphqlEventAdapterConfig,
        GraphqlFetcher,
        create_graphql_event_adapter,
    },
    port::FinalizedBlock,
};
use fuel_streams_domains::{
    blocks::Block,
    transactions::Transaction,
};
use futures::StreamExt;
use std::{
    cmp::Ordering,
    num::NonZeroUsize,
    sync::Arc,
};
use tokio::sync::watch;

pub struct Config {
    pub url: url::Url,
    pub starting_height: BlockHeight,
    pub storage_type: StorageTypeConfig,
    pub batch_size: usize,
    pub blocks_request_batch_size: usize,
    pub blocks_request_concurrency: usize,
    pub pending_blocks: usize,
}

pub struct UninitializedTask {
    config: Config,
    fetcher_factory: Arc<dyn Fn() -> GraphqlFetcher + Send + Sync>,
    shared: SharedState,
}

pub struct Task {
    height: watch::Sender<BlockHeight>,
    /// Factory function to create new GraphqlFetcher instances on reconnection.
    /// We recreate the fetcher on each reconnection to avoid memory leaks from
    /// accumulated background tasks and channels in the external library.
    fetcher_factory: Arc<dyn Fn() -> GraphqlFetcher + Send + Sync>,
    blocks_stream: BoxStream<anyhow::Result<BlockEvent>>,
    /// Disk-based block buffer that writes directly to Avro files
    buffer: DiskBuffer,
    processor: Processor,
    base_asset_id: AssetId,
    batch_size: usize,
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
            fetcher_factory,
            shared,
        } = self;

        let client = FuelClient::new(&config.url)?;
        let base_asset_id = *client
            .chain_info()
            .await?
            .consensus_parameters
            .base_asset_id();

        let processor = Processor::new(config.storage_type).await?;

        let current_height = processor
            .load_latest_height()
            .await?
            .unwrap_or(config.starting_height);
        shared.block_height.send_replace(current_height);

        // Create disk buffer for block accumulation
        let buffer = DiskBuffer::new()?;
        tracing::info!("Using disk buffer for block accumulation");

        let mut task = Task {
            blocks_stream: futures::stream::pending().into_boxed(),
            height: shared.block_height,
            fetcher_factory,
            buffer,
            processor,
            base_asset_id,
            batch_size: config.batch_size,
        };

        task.connect_block_stream().await?;

        Ok(task)
    }
}

impl RunnableTask for Task {
    async fn run(&mut self, watcher: &mut StateWatcher) -> TaskNextAction {
        match self.buffer.len().cmp(&self.batch_size) {
            Ordering::Less => {}
            Ordering::Equal => {
                return TaskNextAction::always_continue(self.post_blocks().await)
            }
            Ordering::Greater => {
                tracing::error!(
                    "Batch size exceeded: {} > {}",
                    self.buffer.len(),
                    self.batch_size
                );
                return TaskNextAction::Stop;
            }
        }

        tokio::select! {
            biased;

            _ = watcher.while_started() => {
                TaskNextAction::Stop
            }

            block = self.blocks_stream.next() => {
                match block {
                    Some(Ok(event)) => {
                        // Get the current height, converting from fuel_streams_types::BlockHeight
                        // to fuel_core_types::fuel_types::BlockHeight if needed
                        let current_height: BlockHeight = if let Some(last_height) = self.buffer.last_height() {
                            (*last_height).into()
                        } else {
                            *self.height.borrow()
                        };

                        let Some(next_height) = current_height.succ() else {
                            tracing::error!("Block height overflowed when processing block event");
                            return TaskNextAction::Stop;
                        };

                        if next_height != *event.header.height() {
                            tracing::warn!(
                                "Received out-of-order block event: expected height {}, got height {}. Reconnecting stream.",
                                next_height,
                                event.header.height()
                            );
                            match self.connect_block_stream().await {
                                Ok(_) => return TaskNextAction::Continue,
                                Err(e) => {
                                    tracing::error!("Failed to reconnect block stream: {e}");
                                    return TaskNextAction::Stop;
                                }
                            }
                        }

                        // Convert event to block and transactions, then buffer
                        match self.append_event_to_buffer(&event) {
                            Ok(_) => TaskNextAction::Continue,
                            Err(e) => {
                                tracing::error!("Failed to buffer block: {e}");
                                TaskNextAction::Stop
                            }
                        }
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
        }
    }

    async fn shutdown(self) -> anyhow::Result<()> {
        Ok(())
    }
}

impl Task {
    async fn connect_block_stream(&mut self) -> anyhow::Result<()> {
        // Reset the buffer when reconnecting
        self.buffer.reset()?;

        let height = *self.height.borrow();
        let next_height = height.succ().ok_or_else(|| {
            anyhow::anyhow!("Block height overflowed when connecting block stream")
        })?;

        // Create a fresh GraphqlFetcher for each connection.
        // This is critical to avoid memory leaks: the external library spawns
        // background tasks and creates large-capacity channels on each
        // blocks_stream_starting_from() call. By creating a new fetcher,
        // we ensure the old tasks/channels are properly abandoned.
        let fetcher = (self.fetcher_factory)();
        tracing::debug!("Created new GraphqlFetcher for stream connection");

        let stream = fetcher
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

    /// Converts a block event to domain types and adds to the buffer
    fn append_event_to_buffer(&mut self, event: &BlockEvent) -> anyhow::Result<()> {
        let block = Block::new(
            &event.header,
            event.consensus.clone(),
            event.transactions.len(),
        )?;

        let transactions: Vec<_> = event
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

        // Add to disk buffer (writes directly to Avro files)
        self.buffer.append(&block, &transactions)?;

        Ok(())
    }

    async fn post_blocks(&mut self) -> anyhow::Result<()> {
        if self.buffer.is_empty() {
            return Ok(())
        }

        // Finalize the buffer and get the file paths for upload.
        // Note: finalize() consumes the writers and FinalizedBatchFiles owns the temp files.
        // If upload fails, the error propagates up and the service will reconnect,
        // resuming from the last successfully saved height.
        let finalized = self
            .buffer
            .finalize()
            .map_err(|err| anyhow::anyhow!("Failed to finalize buffer: {err}"))?;

        // Convert from fuel_streams_types::BlockHeight to fuel_core_types::fuel_types::BlockHeight
        let last_height_u32: u32 = *finalized.last_height;
        let last_height: BlockHeight = last_height_u32.into();

        // Upload the Avro files to storage
        process_finalized_batch(&self.processor, finalized)
            .await
            .map_err(|err| {
                anyhow::anyhow!(
                    "Failed to process batch of blocks and transactions: {err}"
                )
            })?;

        // Only after successful upload do we update state and clear the buffer
        self.processor.save_latest_height(last_height).await?;
        self.height.send_replace(last_height);

        // Clear the buffer now that upload succeeded
        self.buffer.reset()?;

        Ok(())
    }
}

pub fn new_service(config: Config) -> anyhow::Result<ServiceRunner<UninitializedTask>> {
    // Create a shared client that will be reused across all fetcher instances
    let client = Arc::new(FuelClient::new(&config.url)?);

    // Capture config values for the factory closure
    let blocks_request_batch_size = config.blocks_request_batch_size;
    let blocks_request_concurrency = config.blocks_request_concurrency;
    let pending_blocks_limit = config.pending_blocks;

    // Create a factory that produces fresh GraphqlFetcher instances.
    // Each fetcher is created with reduced channel capacities to limit memory usage.
    // The external library spawns background tasks on each stream creation,
    // so we need fresh fetchers on reconnection to avoid task/memory accumulation.
    let fetcher_factory: Arc<dyn Fn() -> GraphqlFetcher + Send + Sync> =
        Arc::new(move || {
            let graphql_config = GraphqlEventAdapterConfig {
                client: client.clone(),
                // The external library creates broadcast channels with this capacity
                // that persist until background tasks terminate.
                heartbeat_capacity: NonZeroUsize::new(100_000).expect("Is not zero; qed"),
                event_capacity: NonZeroUsize::new(100_000).expect("Is not zero; qed"),
                blocks_request_batch_size,
                blocks_request_concurrency,
                pending_blocks_limit,
            };
            create_graphql_event_adapter(graphql_config)
        });

    let (height, _) = watch::channel(config.starting_height);
    let task = UninitializedTask {
        config,
        fetcher_factory,
        shared: SharedState {
            block_height: height,
        },
    };

    Ok(ServiceRunner::new(task))
}

/// Process finalized batch files by uploading to storage.
/// Uploads sequentially to minimize memory usage - each file is streamed
/// directly to S3 without loading into memory.
pub async fn process_finalized_batch(
    processor: &Processor,
    files: FinalizedBatchFiles,
) -> anyhow::Result<()> {
    let first_height = files.first_height;
    let last_height = files.last_height;

    // Upload sequentially to minimize memory usage
    // Each upload streams from disk to S3 without loading into memory
    tracing::info!(
        "Uploading blocks from file: {}",
        files.blocks_path.display()
    );
    processor
        .process_data_from_file(
            first_height,
            last_height,
            &files.blocks_path,
            S3TableName::Blocks,
        )
        .await
        .map_err(|e| anyhow::anyhow!("Failed to upload blocks: {}", e))?;

    tracing::info!(
        "Uploading transactions from file: {}",
        files.transactions_path.display()
    );
    processor
        .process_data_from_file(
            first_height,
            last_height,
            &files.transactions_path,
            S3TableName::Transactions,
        )
        .await
        .map_err(|e| anyhow::anyhow!("Failed to upload transactions: {}", e))?;

    tracing::info!(
        "Uploading receipts from file: {}",
        files.receipts_path.display()
    );
    processor
        .process_data_from_file(
            first_height,
            last_height,
            &files.receipts_path,
            S3TableName::Receipts,
        )
        .await
        .map_err(|e| anyhow::anyhow!("Failed to upload receipts: {}", e))?;

    // FinalizedBatchFiles::drop() will clean up the temp directory
    Ok(())
}

/// Process a batch of blocks and transactions (legacy in-memory method)
/// Kept for backwards compatibility with tests
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
            batch_size: 1,
            blocks_request_batch_size: 10,
            blocks_request_concurrency: 100,
            pending_blocks: 10_000,
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
