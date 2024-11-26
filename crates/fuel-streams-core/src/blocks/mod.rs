pub mod subjects;
pub mod types;

pub use subjects::*;

use super::types::*;
use crate::{StreamEncoder, Streamable};

impl StreamEncoder for Block {}
impl Streamable for Block {
    const NAME: &'static str = "blocks";
    const WILDCARD_LIST: &'static [&'static str] = &[BlocksSubject::WILDCARD];
}

#[cfg(test)]
mod tests {
    use serde_json::{self, json};

    use super::*;

    #[tokio::test]
    async fn test_serialization() {
        let header = BlockHeader {
            application_hash: [0u8; 32].into(),
            consensus_parameters_version: 1,
            da_height: 1000,
            event_inbox_root: [1u8; 32].into(),
            height: 42,
            id: Default::default(),
            message_outbox_root: [3u8; 32].into(),
            message_receipt_count: 10,
            prev_root: [4u8; 32].into(),
            state_transition_bytecode_version: 2,
            time: FuelCoreTai64(1697398400),
            transactions_count: 5,
            transactions_root: [5u8; 32].into(),
            version: BlockHeaderVersion::V1,
        };

        let block = Block {
            consensus: Consensus::default(),
            header: header.clone(),
            height: 42,
            id: Default::default(),
            transactions: vec![], // Always empty for now
            version: BlockVersion::V1,
        };

        let serialized_block =
            serde_json::to_value(&block).expect("Failed to serialize Block");

        let expected_json = json!({
            "consensus": {
                "kind": "Genesis",
                "chain_config_hash": "0000000000000000000000000000000000000000000000000000000000000000",
                "coins_root": "0000000000000000000000000000000000000000000000000000000000000000",
                "contracts_root": "0000000000000000000000000000000000000000000000000000000000000000",
                "messages_root": "0000000000000000000000000000000000000000000000000000000000000000",
                "transactions_root": "0000000000000000000000000000000000000000000000000000000000000000"
            },
            "header": {
                "application_hash": "0000000000000000000000000000000000000000000000000000000000000000",
                "consensus_parameters_version": 1,
                "da_height": 1000,
                "event_inbox_root": "0101010101010101010101010101010101010101010101010101010101010101",
                "height": 42,
                "id": "0000000000000000000000000000000000000000000000000000000000000000",
                "message_outbox_root": "0303030303030303030303030303030303030303030303030303030303030303",
                "message_receipt_count": 10,
                "prev_root": "0404040404040404040404040404040404040404040404040404040404040404",
                "state_transition_bytecode_version": 2,
                "time": [0, 0, 0, 0, 101, 44, 62, 128],
                "transactions_count": 5,
                "transactions_root": "0505050505050505050505050505050505050505050505050505050505050505",
                "version": "V1"
            },
            "height": 42,
            "id": "0000000000000000000000000000000000000000000000000000000000000000",
            "transactions": [],
            "version": "V1"
        });

        assert_eq!(serialized_block, expected_json);
    }
}
