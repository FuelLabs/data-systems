pub mod subjects;
pub mod types;

pub use subjects::*;

use super::types::*;
use crate::{DataEncoder, StreamError, Streamable};

impl DataEncoder for Block {
    type Err = StreamError;
}
impl Streamable for Block {
    const NAME: &'static str = "blocks";
    const WILDCARD_LIST: &'static [&'static str] = &[BlocksSubject::WILDCARD];
}

#[cfg(test)]
mod tests {
    use serde_json::{self, json};

    use super::*;

    #[tokio::test]
    async fn test_block_encode() {
        let block = MockBlock::build(42);
        let encoded = block.encode().await.unwrap();
        let decoded = Block::decode(&encoded).await.unwrap();
        assert_eq!(decoded, block, "Decoded block should match original");
    }

    #[tokio::test]
    async fn test_serialization() {
        let header = BlockHeader {
            application_hash: [0u8; 32].into(),
            consensus_parameters_version: 1,
            da_height: 1000,
            event_inbox_root: [1u8; 32].into(),
            id: Default::default(),
            height: 42,
            message_outbox_root: [3u8; 32].into(),
            message_receipt_count: 10,
            prev_root: [4u8; 32].into(),
            state_transition_bytecode_version: 2,
            time: FuelCoreTai64(1697398400).into(),
            transactions_count: 5,
            transactions_root: [5u8; 32].into(),
            version: BlockHeaderVersion::V1,
        };

        let block = Block {
            consensus: Consensus::default(),
            header: header.clone(),
            height: 42,
            id: Default::default(),
            transaction_ids: vec![],
            version: BlockVersion::V1,
        };

        let serialized_block =
            serde_json::to_value(&block).expect("Failed to serialize Block");

        let expected_json = json!({
            "consensus": {
                "chainConfigHash": "0x0000000000000000000000000000000000000000000000000000000000000000",
                "coinsRoot": "0x0000000000000000000000000000000000000000000000000000000000000000",
                "contractsRoot": "0x0000000000000000000000000000000000000000000000000000000000000000",
                "messagesRoot": "0x0000000000000000000000000000000000000000000000000000000000000000",
                "transactionsRoot": "0x0000000000000000000000000000000000000000000000000000000000000000",
                "type": "Genesis"
            },
            "header": {
                "applicationHash": "0x0000000000000000000000000000000000000000000000000000000000000000",
                "consensusParametersVersion": 1,
                "daHeight": 1000,
                "eventInboxRoot": "0x0101010101010101010101010101010101010101010101010101010101010101",
                "id": "0x0000000000000000000000000000000000000000000000000000000000000000",
                "height": 42,
                "messageOutboxRoot": "0x0303030303030303030303030303030303030303030303030303030303030303",
                "messageReceiptCount": 10,
                "prevRoot": "0x0404040404040404040404040404040404040404040404040404040404040404",
                "stateTransitionBytecodeVersion": 2,
                "time": "1697398400",
                "transactionsCount": 5,
                "transactionsRoot": "0x0505050505050505050505050505050505050505050505050505050505050505",
                "version": "V1"
            },
            "height": 42,
            "id": "0x0000000000000000000000000000000000000000000000000000000000000000",
            "transactionIds": [],
            "version": "V1"
        });

        assert_eq!(serialized_block, expected_json);
    }
}
