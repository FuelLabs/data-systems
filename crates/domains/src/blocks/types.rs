use std::{fmt, str::FromStr};

use fuel_data_parser::DataEncoder;
pub use fuel_streams_types::BlockHeight;
use fuel_streams_types::{fuel_core::*, primitives::*};
use serde::{Deserialize, Serialize};

use crate::infra::ToPacket;

// Block type
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, utoipa::ToSchema)]
pub struct Block {
    pub consensus: Consensus,
    pub header: BlockHeader,
    pub height: BlockHeight,
    pub id: BlockId,
    pub transaction_ids: Vec<TxId>,
    pub version: BlockVersion,
    pub producer: Address,
}

impl DataEncoder for Block {}
impl ToPacket for Block {}

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
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, utoipa::ToSchema)]
#[serde(tag = "type")]
pub enum Consensus {
    Genesis(Genesis),
    PoAConsensus(PoAConsensus),
}

impl From<&Consensus> for ConsensusType {
    fn from(consensus: &Consensus) -> Self {
        match consensus {
            Consensus::Genesis(_) => ConsensusType::Genesis,
            Consensus::PoAConsensus(_) => ConsensusType::PoaConsensus,
        }
    }
}

#[allow(clippy::type_complexity)]
impl Consensus {
    pub fn normalize_all(
        &self,
    ) -> (
        Option<ConsensusType>,
        Option<Bytes32>,
        Option<Bytes32>,
        Option<Bytes32>,
        Option<Bytes32>,
        Option<Bytes32>,
        Option<Signature>,
    ) {
        match self {
            Consensus::Genesis(genesis) => (
                Some(self.into()),
                Some(genesis.chain_config_hash.to_owned()),
                Some(genesis.coins_root.to_owned()),
                Some(genesis.contracts_root.to_owned()),
                Some(genesis.messages_root.to_owned()),
                Some(genesis.transactions_root.to_owned()),
                None,
            ),
            Consensus::PoAConsensus(poa) => (
                Some(self.into()),
                None,
                None,
                None,
                None,
                None,
                Some(poa.signature.to_owned()),
            ),
        }
    }
}

#[derive(
    Clone,
    Debug,
    Default,
    PartialEq,
    Eq,
    Serialize,
    Deserialize,
    utoipa::ToSchema,
)]
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

#[derive(
    Debug, Clone, PartialEq, Eq, Serialize, Deserialize, utoipa::ToSchema,
)]
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
#[derive(
    Debug, Clone, Eq, PartialEq, Serialize, Deserialize, utoipa::ToSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum BlockVersion {
    #[serde(alias = "V1", alias = "v1")]
    V1,
}

impl FromStr for BlockVersion {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().as_str() {
            "V1" => Ok(BlockVersion::V1),
            _ => Err(format!("Unknown BlockVersion: {}", s)),
        }
    }
}

impl fmt::Display for BlockVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            BlockVersion::V1 => "V1",
        };
        write!(f, "{}", s)
    }
}

#[derive(Debug, Clone)]
#[cfg(any(test, feature = "test-helpers"))]
pub struct MockBlock(pub Block);
#[cfg(any(test, feature = "test-helpers"))]
impl MockBlock {
    pub fn build(height: BlockHeight) -> Block {
        use fuel_core_types::blockchain::block::BlockV1;
        let mut block: FuelCoreBlock<FuelCoreTypesTransaction> =
            FuelCoreBlock::V1(BlockV1::default());
        block
            .header_mut()
            .set_block_height(FuelCoreBlockHeight::new(height.into()));
        let txs = (0..50)
            .map(|_| FuelCoreTypesTransaction::default_test_tx())
            .collect::<Vec<_>>();
        *block.transactions_mut() = txs;

        let mut block = Block::new(
            &block,
            Consensus::default(),
            Vec::new(),
            Address::default(),
        );
        let now = chrono::Utc::now();
        block.header.time = BlockTime::from_unix(now.timestamp());
        block
    }

    pub fn random() -> Block {
        use rand::Rng;
        let mut rng = rand::rng();
        let height = rng.random_range(0..u64::MAX);
        Self::build(height.into())
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    use serde_json::{json, Value};

    use super::*;

    #[test]
    fn test_block_version_deserialization() {
        // Test uppercase "V1"
        let uppercase = r#""V1""#;
        let version: BlockVersion = serde_json::from_str(uppercase).unwrap();
        assert_eq!(version, BlockVersion::V1);

        // Test lowercase "v1"
        let lowercase = r#""v1""#;
        let version: BlockVersion = serde_json::from_str(lowercase).unwrap();
        assert_eq!(version, BlockVersion::V1);

        // Test within a JSON object
        let json_obj = json!({
            "version": "V1"
        });
        let parsed: Value = serde_json::from_value(json_obj).unwrap();
        let version: BlockVersion =
            serde_json::from_value(parsed["version"].clone()).unwrap();
        assert_eq!(version, BlockVersion::V1);
    }
}
