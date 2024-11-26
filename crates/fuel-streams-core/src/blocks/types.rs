use crate::types::*;

// Block type
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Block {
    pub consensus: Consensus,
    pub header: BlockHeader,
    pub height: u32,
    pub id: FuelCoreBlockId,
    pub transactions: Vec<Transaction>,
    pub version: BlockVersion,
}

impl Block {
    pub fn new(
        block: &fuel_core_types::blockchain::block::Block,
        consensus: Consensus,
    ) -> Self {
        let header: BlockHeader = block.header().into();
        let height = header.height;
        let id = header.id;

        let version = match block {
            fuel_core_types::blockchain::block::Block::V1(_) => {
                BlockVersion::V1
            }
        };

        Self {
            consensus,
            header,
            height,
            id,
            transactions: Vec::new(),
            version,
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct BlockHeight(String);

impl From<FuelCoreBlockHeight> for BlockHeight {
    fn from(value: FuelCoreBlockHeight) -> Self {
        let height = *value;
        BlockHeight(height.to_string())
    }
}

impl From<u32> for BlockHeight {
    fn from(value: u32) -> Self {
        BlockHeight::from(FuelCoreBlockHeight::from(value))
    }
}

impl std::fmt::Display for BlockHeight {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

// Consensus enum
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Consensus {
    Genesis(Genesis),
    PoAConsensus(FuelCorePoAConsensus),
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
            FuelCoreConsensus::PoA(poa) => Consensus::PoAConsensus(poa),
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
    pub consensus_parameters_version: u32,
    pub da_height: u64,
    pub event_inbox_root: Bytes32,
    pub height: u32,
    pub id: FuelCoreBlockId,
    pub message_outbox_root: Bytes32,
    pub message_receipt_count: u32,
    pub prev_root: Bytes32,
    pub state_transition_bytecode_version: u32,
    pub time: FuelCoreTai64,
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
            consensus_parameters_version: header.consensus_parameters_version,
            da_height: header.da_height.into(),
            event_inbox_root: header.event_inbox_root.into(),
            height: (*header.height()).into(),
            id: header.id(),
            message_outbox_root: header.message_outbox_root.into(),
            message_receipt_count: header.message_receipt_count,
            prev_root: (*header.prev_root()).into(),
            state_transition_bytecode_version: header
                .state_transition_bytecode_version,
            time: header.time(),
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

        Block::new(&block, Consensus::default())
    }
}
