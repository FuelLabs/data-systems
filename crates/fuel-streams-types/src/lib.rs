use fuel_core_types::{
    blockchain::{
        consensus::{
            poa::PoAConsensus,
            Consensus as FuelCoreConsensus,
            Genesis,
        },
        header::BlockHeader,
    },
    fuel_tx::Bytes32,
    tai64::Tai64,
};
use serde::{Deserialize, Serialize};

// Balance type
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Balance {
    pub amount: u64,
    pub asset_id: AssetId,
    pub owner: Address,
}

// Block type
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Block {
    pub consensus: Consensus,
    pub header: Header,
    pub height: u32,
    pub id: BlockId,
    pub transactions: Vec<Transaction>,
    pub version: BlockVersion,
}

impl Block {
    pub fn new(
        block: &fuel_core_types::blockchain::block::Block,
        consensus: Consensus,
    ) -> Self {
        let header: Header = block.header().into();
        let height = header.height;
        let id = header.id.clone();

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

// Consensus enum
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Consensus {
    Genesis(Genesis),
    PoAConsensus(PoAConsensus),
}

impl Default for Consensus {
    fn default() -> Self {
        Consensus::Genesis(Genesis::default())
    }
}

impl From<FuelCoreConsensus> for Consensus {
    fn from(consensus: FuelCoreConsensus) -> Self {
        match consensus {
            FuelCoreConsensus::Genesis(genesis) => Consensus::Genesis(genesis),
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
pub struct Header {
    pub application_hash: Bytes32,
    pub consensus_parameters_version: u32,
    pub da_height: u64,
    pub event_inbox_root: Bytes32,
    pub height: u32,
    pub id: BlockId,
    pub message_outbox_root: Bytes32,
    pub message_receipt_count: u32,
    pub prev_root: Bytes32,
    pub state_transition_bytecode_version: u32,
    pub time: Tai64,
    pub transactions_count: u16,
    pub transactions_root: Bytes32,
    pub version: HeaderVersion,
}

impl From<&BlockHeader> for Header {
    fn from(header: &BlockHeader) -> Self {
        let version = match header {
            BlockHeader::V1(_) => HeaderVersion::V1,
        };

        Header {
            application_hash: *header.application_hash(),
            consensus_parameters_version: header.consensus_parameters_version,
            da_height: header.da_height.into(),
            event_inbox_root: header.event_inbox_root,
            height: (*header.height()).into(),
            id: header.id().to_string(),
            message_outbox_root: header.message_outbox_root,
            message_receipt_count: header.message_receipt_count,
            prev_root: *header.prev_root(),
            state_transition_bytecode_version: header
                .state_transition_bytecode_version,
            time: header.time(),
            transactions_count: header.transactions_count,
            transactions_root: header.transactions_root,
            version,
        }
    }
}

// HeaderVersion enum
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum HeaderVersion {
    V1,
}

// Transaction type
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Transaction {
    pub id: TransactionId,
    pub inputs: Vec<Input>,
    pub outputs: Vec<Output>,
    pub gas_price: u64,
    pub gas_limit: u64,
}

// Input enum
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Input {
    Coin(InputCoin),
    Contract(InputContract),
    Message(InputMessage),
}

// InputCoin type
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]

pub struct InputCoin {
    pub amount: u64,
    pub asset_id: AssetId,
    pub owner: Address,
    pub predicate: HexString,
    pub predicate_data: HexString,
    pub predicate_gas_used: u64,
    pub tx_pointer: TxPointer,
    pub utxo_id: UtxoId,
    pub witness_index: u16,
}

// InputContract type
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]

pub struct InputContract {
    pub balance_root: Bytes32,
    pub contract_id: Bytes32,
    pub state_root: Bytes32,
    pub tx_pointer: TxPointer,
    pub utxo_id: UtxoId,
}

// InputMessage type
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]

pub struct InputMessage {
    pub amount: u64,
    pub data: HexString,
    pub nonce: Nonce,
    pub predicate: HexString,
    pub predicate_data: HexString,
    pub predicate_gas_used: u64,
    pub recipient: Address,
    pub sender: Address,
    pub witness_index: u16,
}

// TxPointer type
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]

pub struct TxPointer {
    pub block_height: u32,
    pub tx_index: u16,
}

// Output enum
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Output {
    Coin(CoinOutput),
    Contract(ContractOutput),
}

// CoinOutput type
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]

pub struct CoinOutput {
    pub amount: u64,
    pub asset_id: AssetId,
    pub to: Address,
}

// ContractOutput type
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]

pub struct ContractOutput {
    pub balance_root: Bytes32,
    pub input_index: u16,
    pub state_root: Bytes32,
}

// Scalar types
pub type Address = String;
pub type AssetId = String;
pub type BlockId = String;
pub type HexString = String;
pub type Nonce = String;
pub type Salt = String;
pub type Tai64Timestamp = String;
pub type UtxoId = String;
pub type TransactionId = String;
