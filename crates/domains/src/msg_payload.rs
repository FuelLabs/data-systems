use std::sync::Arc;

use fuel_core_types::services::executor::Event;
use fuel_data_parser::{
    DataEncoder,
    DataParserError,
};
use fuel_streams_types::{
    Address,
    BlockTimestamp,
    FuelCoreAssetId,
    FuelCoreChainId,
    FuelCoreError,
    TxId,
};
use serde::{
    Deserialize,
    Serialize,
};

use crate::{
    blocks::{
        Block,
        BlockHeight,
        Consensus,
    },
    transactions::Transaction,
};

#[derive(Debug, thiserror::Error)]
pub enum MsgPayloadError {
    #[error("Failed to fetch transaction status: {0}")]
    TransactionStatus(String),
    #[error(transparent)]
    Serialization(#[from] DataParserError),
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

pub type TxItem = (usize, Transaction);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MsgPayload {
    pub block: Block,
    pub transactions: Vec<Transaction>,
    pub metadata: Metadata,
    pub namespace: Option<String>,
    pub events: Vec<Event>,
}

impl DataEncoder for MsgPayload {}

impl MsgPayload {
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

    pub fn block_producer(&self) -> Address {
        self.block.producer.to_owned()
    }

    pub fn block(&self) -> &Block {
        &self.block
    }

    pub fn arc(&self) -> Arc<Self> {
        Arc::new(self.clone())
    }

    pub fn timestamp(&self) -> BlockTimestamp {
        BlockTimestamp::from(&self.block.header)
    }
}

#[cfg(any(test, feature = "test-helpers"))]
pub struct MockMsgPayload(MsgPayload);

#[cfg(any(test, feature = "test-helpers"))]
impl MockMsgPayload {
    pub fn new(height: BlockHeight) -> Self {
        use crate::mocks::*;
        let block = MockBlock::build(height);
        let chain_id = Arc::new(FuelCoreChainId::default());
        let base_asset_id = Arc::new(FuelCoreAssetId::default());
        let block_producer = Arc::new(Address::random());
        let block_height = Arc::new(height);
        let consensus = Arc::new(Consensus::default());
        let transactions = MockTransaction::all();
        let metadata = Metadata {
            chain_id,
            base_asset_id,
            block_producer,
            block_height,
            consensus,
        };

        Self(MsgPayload {
            block,
            transactions,
            metadata,
            namespace: None,
            events: vec![],
        })
    }

    pub fn into_inner(self) -> MsgPayload {
        self.0
    }

    pub fn build(height: BlockHeight, namespace: &str) -> MsgPayload {
        let mut payload = Self::new(height);
        payload.0.namespace = Some(namespace.to_string());
        payload.0
    }

    pub fn with_height(height: BlockHeight) -> Self {
        use crate::mocks::*;
        let mut payload = Self::new(height);
        payload.0.block = MockBlock::build(height);
        payload.0.metadata.block_height = Arc::new(height);
        payload
    }

    pub fn with_transactions(
        height: BlockHeight,
        transactions: Vec<Transaction>,
    ) -> Self {
        let mut payload = Self::new(height);
        payload.0.transactions = transactions;
        payload
    }

    pub fn single_transaction(
        height: BlockHeight,
        r#type: crate::transactions::TransactionType,
    ) -> Self {
        use crate::{
            mocks::*,
            transactions::TransactionType,
        };
        let inputs = MockInput::all();
        let outputs = MockOutput::all();
        let receipts = MockReceipt::all();
        let transaction = match r#type {
            TransactionType::Script => MockTransaction::script(inputs, outputs, receipts),
            TransactionType::Create => MockTransaction::create(inputs, outputs, receipts),
            TransactionType::Mint => MockTransaction::mint(inputs, outputs, receipts),
            TransactionType::Upgrade => {
                MockTransaction::upgrade(inputs, outputs, receipts)
            }
            TransactionType::Upload => MockTransaction::upload(inputs, outputs, receipts),
            TransactionType::Blob => MockTransaction::blob(inputs, outputs, receipts),
        };

        let mut payload = Self::new(height);
        payload.0.transactions = vec![transaction];
        payload
    }
}

#[cfg(any(test, feature = "test-helpers"))]
impl From<&Block> for MockMsgPayload {
    fn from(block: &Block) -> Self {
        MockMsgPayload::new(block.height)
    }
}
