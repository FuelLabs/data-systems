pub use fuel_streams_types::BlockHeight;
use fuel_streams_types::{fuel_core::*, primitives::*};
use serde::{Deserialize, Serialize};
use wrapped_int::*;

// Block type
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Block {
    pub consensus: Consensus,
    pub header: BlockHeader,
    pub height: BlockHeight,
    pub id: BlockId,
    pub transaction_ids: Vec<TxId>,
    pub version: BlockVersion,
    pub producer: Address,
}

impl Block {
    pub fn new(
        block: &fuel_core_types::blockchain::block::Block,
        consensus: Consensus,
        transaction_ids: Vec<TxId>,
        producer: Address,
    ) -> Self {
        let header: BlockHeader = block.header().into();
        let height = header.height;
        let version = match block {
            fuel_core_types::blockchain::block::Block::V1(_) => {
                BlockVersion::V1
            }
        };

        Self {
            consensus,
            header: header.to_owned(),
            height,
            id: header.id,
            transaction_ids,
            version,
            producer,
        }
    }
}

// Consensus enum
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Consensus {
    Genesis(Genesis),
    PoAConsensus(PoAConsensus),
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Genesis {
    pub chain_config_hash: Bytes32,
    pub coins_root: Bytes32,
    pub contracts_root: Bytes32,
    pub messages_root: Bytes32,
    pub transactions_root: Bytes32,
}

impl From<FuelCoreGenesis> for Genesis {
    fn from(genesis: FuelCoreGenesis) -> Self {
        Self {
            chain_config_hash: genesis.chain_config_hash.into(),
            coins_root: genesis.coins_root.into(),
            contracts_root: genesis.contracts_root.into(),
            messages_root: genesis.messages_root.into(),
            transactions_root: genesis.transactions_root.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PoAConsensus {
    pub signature: Signature,
}

impl PoAConsensus {
    pub fn new(signature: Signature) -> Self {
        Self { signature }
    }
}

impl From<FuelCorePoAConsensus> for PoAConsensus {
    fn from(poa: FuelCorePoAConsensus) -> Self {
        Self {
            signature: Signature(poa.signature.into()),
        }
    }
}

impl Default for Consensus {
    fn default() -> Self {
        Consensus::Genesis(Genesis::default())
    }
}

impl From<FuelCoreConsensus> for Consensus {
    fn from(consensus: FuelCoreConsensus) -> Self {
        match consensus {
            FuelCoreConsensus::Genesis(genesis) => {
                Consensus::Genesis(genesis.into())
            }
            FuelCoreConsensus::PoA(poa) => Consensus::PoAConsensus(poa.into()),
            _ => panic!("Unknown consensus type: {:?}", consensus),
        }
    }
}

// BlockVersion enum
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum BlockVersion {
    V1,
}

// Header type
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BlockHeader {
    pub application_hash: Bytes32,
    pub consensus_parameters_version: WrappedU32,
    pub da_height: DaBlockHeight,
    pub event_inbox_root: Bytes32,
    pub id: BlockId,
    pub height: BlockHeight,
    pub message_outbox_root: Bytes32,
    pub message_receipt_count: WrappedU32,
    pub prev_root: Bytes32,
    pub state_transition_bytecode_version: WrappedU32,
    pub time: FuelCoreTai64Timestamp,
    pub transactions_count: u16,
    pub transactions_root: Bytes32,
    pub version: BlockHeaderVersion,
}

impl From<&FuelCoreBlockHeader> for BlockHeader {
    fn from(header: &FuelCoreBlockHeader) -> Self {
        let version = match header {
            FuelCoreBlockHeader::V1(_) => BlockHeaderVersion::V1,
        };

        Self {
            application_hash: (*header.application_hash()).into(),
            consensus_parameters_version: header
                .consensus_parameters_version
                .into(),
            da_height: header.da_height.into(),
            event_inbox_root: header.event_inbox_root.into(),
            id: header.id().into(),
            height: (*header.height()).into(),
            message_outbox_root: header.message_outbox_root.into(),
            message_receipt_count: header.message_receipt_count.into(),
            prev_root: (*header.prev_root()).into(),
            state_transition_bytecode_version: header
                .state_transition_bytecode_version
                .into(),
            time: header.time().into(),
            transactions_count: header.transactions_count,
            transactions_root: header.transactions_root.into(),
            version,
        }
    }
}

// BlockHeaderVersion enum
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum BlockHeaderVersion {
    V1,
}

#[derive(Debug, Clone)]
#[cfg(any(test, feature = "test-helpers"))]
pub struct MockBlock(pub Block);

#[cfg(any(test, feature = "test-helpers"))]
impl MockBlock {
    pub fn build(height: u32) -> Block {
        use fuel_core_types::blockchain::block::BlockV1;
        let mut block: FuelCoreBlock<FuelCoreTransaction> =
            FuelCoreBlock::V1(BlockV1::default());
        block
            .header_mut()
            .set_block_height(FuelCoreBlockHeight::new(height));

        let txs = (0..50)
            .map(|_| FuelCoreTransaction::default_test_tx())
            .collect::<Vec<_>>();
        *block.transactions_mut() = txs;

        Block::new(&block, Consensus::default(), Vec::new(), Address::default())
    }
}
