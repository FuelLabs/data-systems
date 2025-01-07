pub mod blocks;
pub mod inputs;
pub mod logs;
pub mod outputs;
pub mod receipts;
pub mod transactions;
pub mod utxos;

use std::{
    marker::PhantomData,
    sync::{Arc, LazyLock},
};

use displaydoc::Display as DisplayDoc;
use fuel_streams_core::prelude::*;
use fuel_streams_store::{
    record::{DataEncoder, EncoderError, Record, RecordOrder},
    store::StorePacket,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use tokio::task::JoinHandle;

pub static PUBLISHER_MAX_THREADS: LazyLock<usize> = LazyLock::new(|| {
    let available_cpus = num_cpus::get();
    dotenvy::var("PUBLISHER_MAX_THREADS")
        .ok()
        .and_then(|val| val.parse().ok())
        .unwrap_or(available_cpus)
});

pub fn sha256(bytes: &[u8]) -> Bytes32 {
    let mut sha256 = Sha256::new();
    sha256.update(bytes);
    let bytes: [u8; 32] = sha256
        .finalize()
        .as_slice()
        .try_into()
        .expect("Must be 32 bytes");

    bytes.into()
}

#[derive(Debug, thiserror::Error, DisplayDoc)]
pub enum ExecutorError {
    /// Failed to publish: {0}
    PublishFailed(String),
    /// Failed to acquire semaphore: {0}
    SemaphoreError(#[from] tokio::sync::AcquireError),
    /// Failed to serialize block payload: {0}
    Serialization(#[from] serde_json::Error),
    /// Failed to fetch transaction status: {0}
    TransactionStatus(String),
    /// Failed to access offchain database: {0}
    OffchainDatabase(#[from] anyhow::Error),
    /// Failed to join tasks: {0}
    JoinError(#[from] tokio::task::JoinError),
    /// Failed to encode or decode data: {0}
    Encoder(#[from] EncoderError),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Metadata {
    pub chain_id: Arc<FuelCoreChainId>,
    pub base_asset_id: Arc<FuelCoreAssetId>,
    pub block_producer: Arc<Address>,
    pub block_height: Arc<BlockHeight>,
    pub consensus: Arc<Consensus>,
}

impl Metadata {
    pub fn new(
        fuel_core: &Arc<dyn FuelCoreLike>,
        sealed_block: &FuelCoreSealedBlock,
    ) -> Self {
        let block = sealed_block.entity.clone();
        let consensus = sealed_block.consensus.clone();
        let height = *block.header().consensus().height;
        let producer =
            consensus.block_producer(&block.id()).unwrap_or_default();
        Self {
            chain_id: Arc::new(*fuel_core.chain_id()),
            base_asset_id: Arc::new(*fuel_core.base_asset_id()),
            block_producer: Arc::new(producer.into()),
            block_height: Arc::new(height.into()),
            consensus: Arc::new(consensus.into()),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockPayload {
    pub block: Block,
    pub transactions: Vec<Transaction>,
    metadata: Metadata,
}

impl DataEncoder for BlockPayload {
    type Err = ExecutorError;
}

impl BlockPayload {
    pub fn new(
        fuel_core: Arc<dyn FuelCoreLike>,
        sealed_block: &FuelCoreSealedBlock,
        metadata: &Metadata,
    ) -> Result<Self, ExecutorError> {
        let block = sealed_block.entity.clone();
        let txs = Self::txs_from_fuelcore(
            fuel_core.to_owned(),
            sealed_block,
            metadata,
        )?;
        let txs_ids = txs.iter().map(|i| i.id.clone()).collect();
        let block_height = block.header().height();
        let consensus = fuel_core.get_consensus(block_height)?;
        let block = Block::new(&block, consensus.into(), txs_ids);
        Ok(Self {
            block,
            transactions: txs,
            metadata: metadata.to_owned(),
        })
    }

    pub fn tx_ids(&self) -> Vec<Bytes32> {
        self.transactions
            .iter()
            .map(|tx| tx.id.clone())
            .collect::<Vec<_>>()
    }

    pub fn message_id(&self) -> String {
        let height = self.metadata.block_height.clone();
        format!("block_{height}")
    }

    pub fn subject(&self) -> String {
        let producer = self.metadata.block_producer.clone();
        let height = self.metadata.block_height.clone();
        format!("block_submitted.{producer}.{height}")
    }

    pub fn metadata(&self) -> &Metadata {
        &self.metadata
    }

    pub fn block_height(&self) -> u32 {
        self.block.height
    }

    pub fn arc(&self) -> Arc<Self> {
        Arc::new(self.clone())
    }

    pub fn txs_from_fuelcore(
        fuel_core: Arc<dyn FuelCoreLike>,
        sealed_block: &FuelCoreSealedBlock,
        metadata: &Metadata,
    ) -> Result<Vec<Transaction>, ExecutorError> {
        let mut transactions: Vec<Transaction> = vec![];
        let blocks_txs = sealed_block.entity.transactions_vec();
        for tx_item in blocks_txs.iter() {
            let tx_id = tx_item.id(&metadata.chain_id);
            let receipts = fuel_core.get_receipts(&tx_id)?.unwrap_or_default();
            let tx_status = fuel_core.get_tx_status(&tx_id)?;
            let tx_status: TransactionStatus = match tx_status {
                Some(status) => (&status).into(),
                _ => TransactionStatus::None,
            };
            let new_transaction = Transaction::new(
                &tx_id.into(),
                tx_item,
                &tx_status,
                &metadata.base_asset_id,
                &receipts,
            );
            transactions.push(new_transaction);
        }
        Ok(transactions)
    }
}

pub struct Executor<R: Record> {
    pub stream: Arc<Stream<R>>,
    payload: Arc<BlockPayload>,
    semaphore: Arc<tokio::sync::Semaphore>,
    __marker: PhantomData<R>,
}

impl<R: Record> Executor<R> {
    pub fn new(
        payload: &Arc<BlockPayload>,
        stream: &Arc<Stream<R>>,
        semaphore: &Arc<tokio::sync::Semaphore>,
    ) -> Self {
        Self {
            payload: payload.to_owned(),
            stream: stream.to_owned(),
            semaphore: semaphore.to_owned(),
            __marker: PhantomData,
        }
    }

    fn publish(
        &self,
        packet: &StorePacket<R>,
    ) -> JoinHandle<Result<(), ExecutorError>> {
        let wildcard = packet.subject.clone();
        let stream = Arc::clone(&self.stream);
        let permit = Arc::clone(&self.semaphore);

        // TODO: add telemetry back again
        let packet = packet.clone();
        tokio::spawn({
            async move {
                let _permit = permit.acquire().await?;
                match stream.publish(&packet).await {
                    Ok(_) => {
                        tracing::debug!(
                            "Successfully published for stream: {wildcard}"
                        );
                        Ok(())
                    }
                    Err(e) => {
                        tracing::error!("Failed to publish for stream: {wildcard}, error: {e}");
                        Err(ExecutorError::PublishFailed(e.to_string()))
                    }
                }
            }
        })
    }

    pub fn payload(&self) -> Arc<BlockPayload> {
        Arc::clone(&self.payload)
    }

    pub fn metadata(&self) -> &Metadata {
        &self.payload.metadata
    }

    pub fn block(&self) -> &Block {
        &self.payload.block
    }

    pub fn block_height(&self) -> BlockHeight {
        let height = self.block().height;
        BlockHeight::from(height)
    }

    pub fn record_order(&self) -> RecordOrder {
        let height = self.block().height;
        RecordOrder::new(height, None, None)
    }
}
