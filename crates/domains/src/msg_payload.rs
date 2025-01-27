use std::sync::Arc;

use fuel_streams_store::record::{DataEncoder, EncoderError};
use fuel_streams_types::{
    Address,
    FuelCoreAssetId,
    FuelCoreChainId,
    FuelCoreError,
    FuelCoreLike,
    FuelCoreSealedBlock,
    FuelCoreUniqueIdentifier,
    TxId,
};
use serde::{Deserialize, Serialize};

use crate::{
    blocks::{Block, BlockHeight, Consensus},
    transactions::{Transaction, TransactionStatus},
};

#[derive(Debug, thiserror::Error)]
pub enum MsgPayloadError {
    #[error("Failed to fetch transaction status: {0}")]
    TransactionStatus(String),
    #[error(transparent)]
    Serialization(#[from] EncoderError),
    #[error(transparent)]
    FuelCore(#[from] FuelCoreError),
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

pub type TxItem = (usize, Transaction);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MsgPayload {
    pub block: Block,
    pub transactions: Vec<Transaction>,
    pub(crate) metadata: Metadata,
    pub namespace: Option<String>,
}

impl DataEncoder for MsgPayload {
    type Err = MsgPayloadError;
}

impl MsgPayload {
    pub async fn new(
        fuel_core: Arc<dyn FuelCoreLike>,
        sealed_block: &FuelCoreSealedBlock,
        metadata: &Metadata,
    ) -> Result<Self, MsgPayloadError> {
        let (block, producer) =
            fuel_core.get_block_and_producer(sealed_block)?;
        let txs = Self::txs_from_fuelcore(&fuel_core, sealed_block).await?;
        let txs_ids = txs.iter().map(|i| i.id.clone()).collect();
        let block_height = block.header().height();
        let consensus = fuel_core.get_consensus(block_height)?;
        let block = Block::new(&block, consensus.into(), txs_ids, producer);
        Ok(Self {
            block,
            transactions: txs,
            metadata: metadata.to_owned(),
            namespace: None,
        })
    }

    pub fn with_namespace(mut self, namespace: &str) -> Self {
        self.namespace = Some(namespace.to_string());
        self
    }

    pub fn tx_ids(&self) -> Vec<TxId> {
        self.transactions
            .iter()
            .map(|tx| tx.id.clone())
            .collect::<Vec<_>>()
    }

    pub fn metadata(&self) -> &Metadata {
        &self.metadata
    }

    pub fn block_height(&self) -> BlockHeight {
        self.block.height
    }

    pub fn block(&self) -> &Block {
        &self.block
    }

    pub fn arc(&self) -> Arc<Self> {
        Arc::new(self.clone())
    }

    pub async fn txs_from_fuelcore(
        fuel_core: &Arc<dyn FuelCoreLike>,
        sealed_block: &FuelCoreSealedBlock,
    ) -> Result<Vec<Transaction>, MsgPayloadError> {
        let mut transactions: Vec<Transaction> = vec![];
        let blocks_txs = sealed_block.entity.transactions_vec();
        for tx in blocks_txs.iter() {
            let tx = Self::tx_from_fuel_core(fuel_core, tx).await?;
            transactions.push(tx);
        }
        Ok(transactions)
    }

    pub async fn tx_from_fuel_core(
        fuel_core: &Arc<dyn FuelCoreLike>,
        tx: &fuel_tx::Transaction,
    ) -> Result<Transaction, MsgPayloadError> {
        let chain_id = fuel_core.chain_id();
        let base_asset_id = fuel_core.base_asset_id();
        let tx_id = tx.id(chain_id);
        let tx_status = Self::retrieve_tx_status(fuel_core, &tx_id, 0).await?;
        let receipts = fuel_core.get_receipts(&tx_id)?.unwrap_or_default();
        Ok(Transaction::new(
            &tx_id.into(),
            tx,
            &tx_status,
            base_asset_id,
            &receipts,
        ))
    }

    async fn retrieve_tx_status(
        fuel_core: &Arc<dyn FuelCoreLike>,
        tx_id: &fuel_tx::Bytes32,
        attempts: u8,
    ) -> Result<TransactionStatus, MsgPayloadError> {
        if attempts > 5 {
            return Err(MsgPayloadError::TransactionStatus(tx_id.to_string()))
        }
        let tx_status = fuel_core.get_tx_status(tx_id)?;
        match tx_status {
            Some(status) => Ok((&status).into()),
            _ => {
                tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
                Box::pin(Self::retrieve_tx_status(
                    fuel_core,
                    tx_id,
                    attempts + 1,
                ))
                .await
            }
        }
    }
}

#[cfg(any(test, feature = "test-helpers"))]
pub struct MockMsgPayload;

#[cfg(any(test, feature = "test-helpers"))]
impl MockMsgPayload {
    pub fn build(height: u32) -> MsgPayload {
        use crate::mocks::*;
        let block = MockBlock::build(height);
        let chain_id = Arc::new(FuelCoreChainId::default());
        let base_asset_id = Arc::new(FuelCoreAssetId::default());
        let block_producer = Arc::new(Address::default());
        let block_height = Arc::new(BlockHeight::from(1_u32));
        let consensus = Arc::new(Consensus::default());
        let transactions = MockTransaction::all();
        let metadata = Metadata {
            chain_id,
            base_asset_id,
            block_producer,
            block_height,
            consensus,
        };

        MsgPayload {
            block,
            transactions,
            metadata,
            namespace: None,
        }
    }

    pub fn with_height(height: u32) -> MsgPayload {
        use crate::mocks::*;
        let mut payload = Self::build(height);
        payload.block = MockBlock::build(height);
        payload.metadata.block_height = Arc::new(BlockHeight::from(height));
        payload
    }

    pub fn with_transactions(
        height: u32,
        transactions: Vec<Transaction>,
    ) -> MsgPayload {
        let mut payload = Self::build(height);
        payload.transactions = transactions;
        payload
    }

    pub fn single_transaction(
        height: u32,
        tx_type: crate::transactions::TransactionKind,
    ) -> MsgPayload {
        use crate::{mocks::*, transactions::TransactionKind};
        let inputs = MockInput::all();
        let outputs = MockOutput::all();
        let receipts = MockReceipt::all();
        let transaction = match tx_type {
            TransactionKind::Script => {
                MockTransaction::script(inputs, outputs, receipts)
            }
            TransactionKind::Create => {
                MockTransaction::create(inputs, outputs, receipts)
            }
            TransactionKind::Mint => {
                MockTransaction::mint(inputs, outputs, receipts)
            }
            TransactionKind::Upgrade => {
                MockTransaction::upgrade(inputs, outputs, receipts)
            }
            TransactionKind::Upload => {
                MockTransaction::upload(inputs, outputs, receipts)
            }
            TransactionKind::Blob => {
                MockTransaction::blob(inputs, outputs, receipts)
            }
        };

        let mut payload = Self::build(height);
        payload.transactions = vec![transaction];
        payload
    }
}
