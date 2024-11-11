use std::sync::Arc;

use async_nats::{jetstream::stream::State as StreamState, RequestErrorKind};
use fuel_core::database::database_description::DatabaseHeight;
use fuel_core_importer::ImporterResult;
use fuel_core_types::fuel_tx::Output;
use fuel_streams::types::Log;
use fuel_streams_core::prelude::*;
use futures::{stream::FuturesUnordered, StreamExt};
use rayon::prelude::*;
use thiserror::Error;
use tokio::sync::{broadcast::error::RecvError, Semaphore};

use crate::{
    blocks,
    packets::{PublishError, PublishOpts},
    publisher_shutdown::{ShutdownToken, GRACEFUL_SHUTDOWN_TIMEOUT},
    telemetry::Telemetry,
    transactions,
    FuelCoreLike,
    PUBLISHER_MAX_THREADS,
};

#[derive(Clone, Debug)]
/// Streams we currently support publishing to.
pub struct Streams {
    pub transactions: Stream<Transaction>,
    pub blocks: Stream<Block>,
    pub inputs: Stream<Input>,
    pub outputs: Stream<Output>,
    pub receipts: Stream<Receipt>,
    pub utxos: Stream<Utxo>,
    pub logs: Stream<Log>,
}

impl Streams {
    pub async fn new(nats_client: &NatsClient) -> Self {
        Self {
            transactions: Stream::<Transaction>::new(nats_client).await,
            blocks: Stream::<Block>::new(nats_client).await,
            inputs: Stream::<Input>::new(nats_client).await,
            outputs: Stream::<Output>::new(nats_client).await,
            receipts: Stream::<Receipt>::new(nats_client).await,
            utxos: Stream::<Utxo>::new(nats_client).await,
            logs: Stream::<Log>::new(nats_client).await,
        }
    }

    pub fn subjects_wildcards(&self) -> &[&'static str] {
        &[
            TransactionsSubject::WILDCARD,
            BlocksSubject::WILDCARD,
            InputsByIdSubject::WILDCARD,
            InputsCoinSubject::WILDCARD,
            InputsMessageSubject::WILDCARD,
            InputsContractSubject::WILDCARD,
            ReceiptsLogSubject::WILDCARD,
            ReceiptsBurnSubject::WILDCARD,
            ReceiptsByIdSubject::WILDCARD,
            ReceiptsCallSubject::WILDCARD,
            ReceiptsMintSubject::WILDCARD,
            ReceiptsPanicSubject::WILDCARD,
            ReceiptsReturnSubject::WILDCARD,
            ReceiptsRevertSubject::WILDCARD,
            ReceiptsLogDataSubject::WILDCARD,
            ReceiptsTransferSubject::WILDCARD,
            ReceiptsMessageOutSubject::WILDCARD,
            ReceiptsReturnDataSubject::WILDCARD,
            ReceiptsTransferOutSubject::WILDCARD,
            ReceiptsScriptResultSubject::WILDCARD,
            UtxosSubject::WILDCARD,
            LogsSubject::WILDCARD,
        ]
    }

    pub async fn get_consumers_and_state(
        &self,
    ) -> Result<Vec<(String, Vec<String>, StreamState)>, RequestErrorKind> {
        Ok(vec![
            self.transactions.get_consumers_and_state().await?,
            self.blocks.get_consumers_and_state().await?,
            self.inputs.get_consumers_and_state().await?,
            self.receipts.get_consumers_and_state().await?,
            self.utxos.get_consumers_and_state().await?,
            self.logs.get_consumers_and_state().await?,
        ])
    }

    #[cfg(feature = "test-helpers")]
    pub async fn is_empty(&self) -> bool {
        self.blocks.is_empty(BlocksSubject::WILDCARD).await
            && self
                .transactions
                .is_empty(TransactionsSubject::WILDCARD)
                .await
    }
}

#[derive(Error, Debug)]
pub enum PublishBlockError {
    #[error("Task execution error: {0}")]
    TaskExecution(String),
    #[error("Publish error: {0}")]
    Publish(#[from] PublishError),
}

#[allow(dead_code)]
#[derive(Clone)]
pub struct Publisher {
    pub fuel_core: Arc<dyn FuelCoreLike>,
    pub nats_client: NatsClient,
    pub streams: Arc<Streams>,
    pub telemetry: Arc<Telemetry>,
}

impl Publisher {
    pub async fn new(
        fuel_core: Arc<dyn FuelCoreLike>,
        nats_url: &str,
        telemetry: Arc<Telemetry>,
    ) -> anyhow::Result<Self> {
        let nats_client_opts = NatsClientOpts::admin_opts(nats_url);
        let nats_client = NatsClient::connect(&nats_client_opts).await?;
        let streams = Arc::new(Streams::new(&nats_client).await);

        telemetry.record_streams_count(
            fuel_core.chain_id(),
            streams.subjects_wildcards().len(),
        );

        Ok(Publisher {
            fuel_core,
            streams,
            nats_client,
            telemetry,
        })
    }

    #[cfg(feature = "test-helpers")]
    pub async fn default(
        nats_client: &NatsClient,
        fuel_core: Arc<dyn FuelCoreLike>,
    ) -> anyhow::Result<Self> {
        Ok(Publisher {
            fuel_core,
            streams: Arc::new(Streams::new(nats_client).await),
            nats_client: nats_client.clone(),
            telemetry: Telemetry::new().await?,
        })
    }

    #[cfg(feature = "test-helpers")]
    pub fn get_streams(&self) -> &Streams {
        &self.streams
    }

    async fn publish_block_data(
        &self,
        result: ImporterResult,
    ) -> anyhow::Result<()> {
        let (block, block_producer) =
            self.fuel_core.get_block_and_producer(&result.sealed_block);
        self.publish(&block, &block_producer).await?;
        Ok(())
    }

    async fn shutdown_services_with_timeout(&self) -> anyhow::Result<()> {
        tokio::time::timeout(GRACEFUL_SHUTDOWN_TIMEOUT, async {
            Publisher::flush_await_all_streams(&self.nats_client).await;
            self.fuel_core.stop().await;
        })
        .await?;

        Ok(())
    }

    async fn flush_await_all_streams(nats_client: &NatsClient) {
        tracing::info!("Flushing in-flight messages to nats ...");
        match nats_client.nats_client.flush().await {
            Ok(_) => {
                tracing::info!("Flushed all streams successfully!");
            }
            Err(e) => {
                tracing::error!("Failed to flush all streams: {:?}", e);
            }
        }
    }

    pub async fn run(
        &self,
        shutdown_token: ShutdownToken,
    ) -> anyhow::Result<()> {
        let last_published_block = self
            .streams
            .blocks
            .get_last_published(BlocksSubject::WILDCARD)
            .await?;
        let last_published_height = last_published_block
            .map(|block| block.header().height().as_u64())
            .unwrap_or(0);

        // Catch up the streams with the FuelCore
        if let Some(latest_fuel_core_height) =
            self.fuel_core.get_latest_block_height()?
        {
            if latest_fuel_core_height > last_published_height + 1 {
                tracing::warn!("Missing blocks: last block height in Node={latest_fuel_core_height}, last published block height={last_published_height}");
            }

            // Republish the last block for idempotency
            let mut height = last_published_height;
            while height <= latest_fuel_core_height {
                tokio::select! {
                    shutdown =  shutdown_token.wait_for_shutdown() => {
                        if shutdown {
                            tracing::info!("Shutdown signal received during historical blocks processing. Last published block height {height}");
                            self.shutdown_services_with_timeout().await?;
                            return Ok(());
                        }
                    },
                    (result, block_producer) = async {
                        let sealed_block = self
                            .fuel_core
                            .get_sealed_block_by_height(height as u32);

                        let (block, block_producer) =
                            self.fuel_core.get_block_and_producer(&sealed_block);

                        (self.publish(&block, &block_producer).await, block_producer.clone())
                    } => {
                        if let Err(err) = result {
                            tracing::warn!("Failed to publish block data: {}", err);
                        }
                        self.telemetry.record_failed_publishing(self.fuel_core.chain_id(), &block_producer);
                        height += 1;
                    }
                }
            }
        }

        // publish subscribed data
        loop {
            let fuel_core = self.fuel_core.clone();
            let mut blocks_subscription = fuel_core.blocks_subscription();

            tokio::select! {
                result = blocks_subscription.recv() => {
                    match result {
                        Ok(result) => {
                            self.publish_block_data(result).await?;
                        }
                        Err(RecvError::Closed) | Err(RecvError::Lagged(_)) => {
                            // The sender has been dropped or has lagged, exit the loop
                            break;
                        }
                    }
                }
                shutdown = shutdown_token.wait_for_shutdown()  => {
                    if shutdown {
                        self.shutdown_services_with_timeout().await?;
                        break;
                    }
                }
            }
        }

        Ok(())
    }

    async fn publish(
        &self,
        block: &Block<Transaction>,
        block_producer: &Address,
    ) -> anyhow::Result<()> {
        let start_time = std::time::Instant::now();
        let fuel_core = &*self.fuel_core;
        let semaphore = Arc::new(Semaphore::new(*PUBLISHER_MAX_THREADS));
        let chain_id = Arc::new(*self.fuel_core.chain_id());
        let block_producer = Arc::new(block_producer.clone());
        let block_height = block.header().consensus().height;
        let txs = block.transactions();

        let streams = (*self.streams).clone();
        let block_stream = Arc::new(streams.blocks.to_owned());
        let opts = &Arc::new(PublishOpts {
            semaphore,
            chain_id,
            block_producer: Arc::clone(&block_producer),
            block_height: Arc::new(block_height.into()),
            telemetry: self.telemetry.clone(),
        });

        let tasks =
            transactions::publish_all_tasks(txs, streams, opts, fuel_core)
                .into_iter()
                .chain(std::iter::once(blocks::publish_task(
                    block,
                    block_stream,
                    opts,
                )))
                .collect::<FuturesUnordered<_>>();

        let errors: Vec<PublishBlockError> = tasks
            .filter_map(|res| self.handle_task_result(res))
            .collect()
            .await;

        if !errors.is_empty() {
            let error_message = errors
                .par_iter()
                .map(|e| e.to_string())
                .collect::<Vec<_>>()
                .join(", ");

            Err(anyhow::anyhow!(
                "Errors occurred during block publishing: {}",
                error_message
            ))
        } else {
            let elapsed = start_time.elapsed();
            tracing::info!(
                "Published streams for BlockHeight: {} in {:?}",
                *block_height,
                elapsed
            );

            Ok(())
        }
    }

    async fn handle_task_result(
        &self,
        result: Result<Result<(), PublishError>, tokio::task::JoinError>,
    ) -> Option<PublishBlockError> {
        match result {
            Ok(Ok(())) => None,
            Ok(Err(publish_error)) => {
                tracing::error!("Publish error: {:?}", publish_error);
                Some(PublishBlockError::Publish(publish_error))
            }
            Err(join_error) => {
                tracing::error!("Task join error: {:?}", join_error);
                Some(PublishBlockError::TaskExecution(join_error.to_string()))
            }
        }
    }
}
