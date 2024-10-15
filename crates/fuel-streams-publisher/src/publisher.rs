use std::{sync::Arc, time::Instant};

use async_nats::{jetstream::stream::State as StreamState, RequestErrorKind};
use fuel_core::database::database_description::DatabaseHeight;
use fuel_core_bin::FuelService;
use fuel_core_importer::ImporterResult;
use fuel_core_types::fuel_tx::{field::ScriptData, Output};
use fuel_streams::types::{ChainId, Log, UniqueIdentifier};
use fuel_streams_core::{
    blocks::BlocksSubject,
    inputs::{
        InputsByIdSubject,
        InputsCoinSubject,
        InputsContractSubject,
        InputsMessageSubject,
    },
    logs::LogsSubject,
    nats::{NatsClient, NatsClientOpts},
    prelude::*,
    receipts::*,
    transactions::{TransactionExt, TransactionsSubject},
    types::{Address, Block, Input, Receipt, Transaction},
    utxos::{types::Utxo, UtxosSubject},
    Stream,
};
use futures_util::{
    future::{try_join_all, BoxFuture},
    FutureExt,
};
use tokio::sync::broadcast::error::RecvError;
use tracing::warn;

use crate::{
    blocks,
    inputs,
    logs,
    metrics::PublisherMetrics,
    outputs,
    receipts,
    sha256,
    shutdown::{StopHandle, GRACEFUL_SHUTDOWN_TIMEOUT},
    transactions,
    utxos,
    FuelCore,
    FuelCoreLike,
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

#[allow(dead_code)]
/// TODO: Remove right after using chain_id and base_asset_id to publish
/// TransactionsById subject
pub struct Publisher {
    fuel_service: Arc<FuelService>,
    fuel_core: Box<dyn FuelCoreLike>,
    metrics: Arc<PublisherMetrics>,
    nats_client: NatsClient,
    streams: Arc<Streams>,
}

impl Publisher {
    pub async fn new(
        fuel_service: Arc<FuelService>,
        nats_url: &str,
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
        })
    }

    pub async fn flush_await_all_streams(&self) -> anyhow::Result<()> {
        let streams = [
            self.streams.blocks.flush_await(&self.nats_client).boxed(),
            self.streams
                .transactions
                .flush_await(&self.nats_client)
                .boxed(),
        ];
        try_join_all(streams).await?;
        Ok(())
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
            tracing::info!("Flushing in-flight messages to nats ...");
            match self.flush_await_all_streams().await {
                Ok(_) => tracing::info!("Flushed in-flight messages to nats"),
                Err(e) => tracing::error!(
                    "Flushing in-flight messages to nats failed: {:?}",
                    e
                ),
            }

            tracing::info!("Stopping fuel core ...");
            match self
                .fuel_service
                .send_stop_signal_and_await_shutdown()
                .await
            {
                Ok(state) => {
                    tracing::info!("Stopped fuel core. Status = {:?}", state)
                }
                Err(e) => tracing::error!("Stopping fuel core failed: {:?}", e),
            }
        })
        .await?;

        Ok(())
    }

    pub async fn run(mut self) -> anyhow::Result<Self> {
        let mut stop_handle = StopHandle::new();
        stop_handle.spawn_signal_listener();

        let last_published_block = self
            .streams
            .blocks
            .get_last_published(BlocksSubject::WILDCARD)
            .await?;
        let last_published_height = last_published_block
            .map(|block| block.header().height().as_u64())
            .unwrap_or(0);
        let next_height_to_publish = last_published_height + 1;

        // Catch up the streams with the FuelCore
        if let Some(latest_fuel_core_height) =
            self.fuel_core.get_latest_block_height()?
        {
            if latest_fuel_core_height > last_published_height + 1 {
                warn!("Missing blocks: last block height in Node={latest_fuel_core_height}, last published block height={last_published_height}");
            }

            // publish historical data if needed
            let mut height = next_height_to_publish;
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
                        self.metrics.total_failed_messages.with_label_values(&[&self.fuel_core.chain_id().to_string(), &block_producer.to_string()]).inc();
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
        let transactions = block.transactions();
        let block_height = block.header().consensus().height;

        let mut publishing_tasks = vec![blocks::publish(
            &self.metrics,
            &*self.fuel_core,
            &self.streams.blocks,
            block,
            block_producer,
        )
        .boxed()];

        for (transaction_index, transaction) in transactions.iter().enumerate()
        {
            let chain_id = self.fuel_core.chain_id();
            let tx_id = transaction.id(chain_id);
            let receipts = self.fuel_core.get_receipts(&tx_id)?;

            publishing_tasks.extend(
                self.build_transaction_related_publishing_tasks(
                    transaction_index,
                    transaction,
                    &(tx_id.into()),
                    receipts.clone(),
                    block_height.into(),
                    block_producer,
                    chain_id,
                    None,
                    None,
                ),
            );

            // Publish predicates
            for input in transaction.inputs() {
                if let Some((
                    predicate_bytecode,
                    _predicate_data,
                    _predicate_gas_used,
                )) = input.predicate()
                {
                    let predicate_tag = sha256(predicate_bytecode);
                    let predicate_publishing_tasks = self
                        .build_transaction_related_publishing_tasks(
                            transaction_index,
                            transaction,
                            &(tx_id.into()),
                            receipts.clone(),
                            block_height.into(),
                            block_producer,
                            chain_id,
                            Some(predicate_tag),
                            None,
                        );

                    publishing_tasks.extend(predicate_publishing_tasks);
                }
            }

            // Publish scripts
            if let Some(script) = transaction.as_script() {
                let script_tag = sha256(script.script_data());
                let script_publishing_tasks = self
                    .build_transaction_related_publishing_tasks(
                        transaction_index,
                        transaction,
                        &(tx_id.into()),
                        receipts.clone(),
                        block_height.into(),
                        block_producer,
                        chain_id,
                        None,
                        Some(script_tag),
                    );

                publishing_tasks.extend(script_publishing_tasks);
            }
        }

        let start_time = Instant::now();
        try_join_all(publishing_tasks).await?;
        tracing::info!(
            "Published streams for BlockHeight: {block_height} in {:?} ms",
            start_time.elapsed().as_millis()
        );

        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    fn build_transaction_related_publishing_tasks<'a>(
        &'a self,
        transaction_index: usize,
        transaction: &'a Transaction,
        tx_id: &Bytes32,
        receipts: Option<Vec<Receipt>>,
        block_height: BlockHeight,
        block_producer: &'a Address,
        chain_id: &'a ChainId,
        predicate_tag: Option<Bytes32>,
        script_tag: Option<Bytes32>,
    ) -> Vec<BoxFuture<'a, anyhow::Result<()>>> {
        vec![
            transactions::publish(
                &self.streams.transactions,
                (transaction_index, transaction),
                &*self.fuel_core,
                block_height.clone(),
                &self.metrics,
                block_producer,
                predicate_tag.clone(),
                script_tag.clone(),
            )
            .boxed(),
            receipts::publish(
                &self.streams.receipts,
                receipts.clone(),
                tx_id.clone(),
                *chain_id,
                &self.metrics,
                block_producer,
                predicate_tag.clone(),
                script_tag.clone(),
            )
            .boxed(),
            logs::publish(
                &self.streams.logs,
                receipts.clone(),
                tx_id.clone(),
                chain_id,
                block_height.clone(),
                &self.metrics,
                block_producer,
                predicate_tag.clone(),
                script_tag.clone(),
            )
            .boxed(),
            inputs::publish(
                &self.streams.inputs,
                transaction,
                chain_id,
                &self.metrics,
                block_producer,
                predicate_tag.clone(),
                script_tag.clone(),
            )
            .boxed(),
            outputs::publish(
                &self.streams.outputs,
                chain_id,
                transaction,
                &self.metrics,
                block_producer,
                predicate_tag.clone(),
                script_tag.clone(),
            )
            .boxed(),
            utxos::publish(
                &self.metrics,
                &self.streams.utxos,
                &*self.fuel_core,
                transaction,
                tx_id.clone(),
                chain_id,
                block_producer,
                predicate_tag,
                script_tag,
            )
            .boxed(),
        ]
    }
}
