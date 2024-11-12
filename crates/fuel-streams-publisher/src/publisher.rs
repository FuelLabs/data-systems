use std::sync::Arc;

use anyhow::Context;
use async_nats::{jetstream::stream::State as StreamState, RequestErrorKind};
use fuel_core::database::database_description::DatabaseHeight;
use fuel_core_bin::FuelService;
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
    elastic::{create_elasticsearch_instance, ElasticSearch},
    metrics::PublisherMetrics,
    packets::{PublishError, PublishOpts},
    shutdown::{StopHandle, GRACEFUL_SHUTDOWN_TIMEOUT},
    transactions,
    FuelCore,
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
            self.outputs.get_consumers_and_state().await?,
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
/// TODO: Remove right after using chain_id and base_asset_id to publish
/// TransactionsById subject
pub struct Publisher {
    fuel_service: Arc<FuelService>,
    fuel_core: Box<dyn FuelCoreLike>,
    metrics: Arc<PublisherMetrics>,
    elastic_logger: Option<Arc<ElasticSearch>>,
    nats_client: NatsClient,
    streams: Arc<Streams>,
}

impl Publisher {
    pub async fn new(
        fuel_service: Arc<FuelService>,
        nats_url: &str,
        use_elastic_logging: bool,
        metrics: Arc<PublisherMetrics>,
        streams: Arc<Streams>,
    ) -> anyhow::Result<Self> {
        let nats_client_opts = NatsClientOpts::admin_opts(nats_url);
        let nats_client = NatsClient::connect(&nats_client_opts).await?;

        let blocks_subscription = fuel_service
            .shared
            .block_importer
            .block_importer
            .subscribe();
        let fuel_core_database = fuel_service.shared.database.clone();

        let chain_config =
            fuel_service.shared.config.snapshot_reader.chain_config();
        let chain_id = chain_config.consensus_parameters.chain_id();

        let fuel_core =
            FuelCore::new(fuel_core_database, blocks_subscription, chain_id)
                .await;

        metrics
            .total_subs
            .with_label_values(&[&chain_id.to_string()])
            .set(streams.subjects_wildcards().len() as i64);

        Ok(Publisher {
            fuel_service,
            fuel_core,
            streams,
            metrics,
            nats_client,
            elastic_logger: if use_elastic_logging {
                Some(Arc::new(create_elasticsearch_instance().await?))
            } else {
                None
            },
        })
    }

    #[cfg(feature = "test-helpers")]
    pub async fn default_with_publisher(
        nats_client: &NatsClient,
        fuel_core: Box<dyn FuelCoreLike>,
    ) -> anyhow::Result<Self> {
        use fuel_core::service::Config;

        let fuel_srv =
            FuelService::new_node(Config::local_node()).await.unwrap();

        Ok(Publisher {
            fuel_service: Arc::new(fuel_srv),
            fuel_core,
            streams: Arc::new(Streams::new(nats_client).await),
            metrics: Arc::new(PublisherMetrics::random()),
            nats_client: nats_client.clone(),
            elastic_logger: None,
        })
    }

    #[cfg(feature = "test-helpers")]
    pub fn get_streams(&self) -> &Streams {
        &self.streams
    }

    // fn set_panic_hook(&mut self) {
    //     let nats_client = self.nats_client.clone();
    //     let fuel_service = Arc::clone(&self.fuel_service);
    //     std::panic::set_hook(Box::new(move |panic_info| {
    //         let payload = panic_info
    //             .payload()
    //             .downcast_ref::<&str>()
    //             .unwrap_or(&"Unknown panic");
    //         tracing::error!("Publisher panicked with a message: {:?}", payload);
    //         let handle = tokio::runtime::Handle::current();
    //         let nats_client = nats_client.clone();
    //         let fuel_service = Arc::clone(&fuel_service);
    //         handle.spawn(async move {
    //             Publisher::flush_await_all_streams(&nats_client).await;
    //             Publisher::stop_fuel_service(&fuel_service).await;
    //             std::process::exit(1);
    //         });
    //     }));
    // }

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
            Publisher::stop_fuel_service(&self.fuel_service).await;
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

    async fn stop_fuel_service(fuel_service: &Arc<FuelService>) {
        if matches!(
            fuel_service.state(),
            fuel_core_services::State::Stopped
                | fuel_core_services::State::Stopping
                | fuel_core_services::State::StoppedWithError(_)
                | fuel_core_services::State::NotStarted
        ) {
            return;
        }

        tracing::info!("Stopping fuel core ...");
        match fuel_service.send_stop_signal_and_await_shutdown().await {
            Ok(state) => {
                tracing::info!("Stopped fuel core. Status = {:?}", state)
            }
            Err(e) => tracing::error!("Stopping fuel core failed: {:?}", e),
        }
    }

    pub async fn run(mut self) -> anyhow::Result<Self> {
        if let Some(elastic_logger) = self.elastic_logger.as_ref() {
            tracing::info!(
                "Elastic logger connection live? {:?}",
                elastic_logger.get_conn().check_alive().unwrap_or_default()
            );
            elastic_logger
                .get_conn()
                .ping()
                .await
                .context("Error pinging elastisearch connection")?;
            tracing::info!("Elastic logger pinged successfully!");
        }

        let mut stop_handle = StopHandle::new();
        stop_handle.spawn_signal_listener();
        // self.set_panic_hook();

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

            let latest_fuel_block_time_unix = self
                .fuel_core
                .get_sealed_block_time_by_height(latest_fuel_core_height as u32)
                .to_unix();

            // Republish the last block. Why? We publish multiple data from the same
            // block and it is not atomic. If the publisher is abruptly stopped, the
            // block itself might be published, but the transactions and receipts
            // might not be. Publishing is idempotent, so republishing the last block
            // is safe and it's a simple way to ensure we don't miss any data. Thus,
            // this loop will run at least once, to republish the last block, and
            // then it will also republish any missing blocks.
            let mut height = last_published_height;
            while height <= latest_fuel_core_height {
                tokio::select! {
                    shutdown = stop_handle.wait_for_signal() => {
                        if shutdown {
                            tracing::info!("Shutdown signal received during historical blocks processing. Last published block height {height}");
                            self.shutdown_services_with_timeout().await?;
                            return Ok(self);
                        }
                    },
                    (result, block_producer) = async {

                        let fuel_block_time_unix = self
                        .fuel_core
                        .get_sealed_block_time_by_height(height as u32)
                        .to_unix();

                        if fuel_block_time_unix < latest_fuel_block_time_unix - (FUEL_BLOCK_TIME_SECS  * MAX_RETENTION_BLOCKS) as i64 {
                            // Skip publishing for this block and move to the next height
                            return (Ok(()), None);
                        }

                        let sealed_block = self
                            .fuel_core
                            .get_sealed_block_by_height(height as u32);

                        let (block, block_producer) =
                            self.fuel_core.get_block_and_producer(&sealed_block);

                        (self.publish(&block, &block_producer).await, Some(block_producer.clone()))
                    } => {
                        if let (Err(err), Some(block_producer)) = (result, block_producer) {
                            tracing::warn!("Failed to publish block data: {}", err);
                            self.metrics.total_failed_messages.with_label_values(&[
                                &self.fuel_core.chain_id().to_string(),
                                &block_producer.to_string()
                            ]).inc();
                        }
                        // Increment the height after processing
                        height += 1;
                    }
                }
            }
        }

        // publish subscribed data
        loop {
            tokio::select! {
                result = self.fuel_core.blocks_subscription().recv() => {
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
                shutdown = stop_handle.wait_for_signal() => {
                    if shutdown {
                        self.shutdown_services_with_timeout().await?;
                        break;
                    }
                }
            }
        }

        Ok(self)
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
            metrics: Arc::clone(&self.metrics),
            block_producer: Arc::clone(&block_producer),
            block_height: Arc::new(block_height.into()),
            elastic_logger: self.elastic_logger.to_owned(),
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
