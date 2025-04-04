use apache_avro::AvroSchema;
use fuel_streams_domains::blocks::{Block, Consensus};
use serde::{Deserialize, Serialize};

use super::AvroTransaction;

#[derive(
    Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize, AvroSchema,
)]
#[serde(rename_all = "camelCase")]
pub struct Genesis {
    #[avro(rename = "chainConfigHash")]
    pub chain_config_hash: Vec<u8>,
    #[avro(rename = "coinsRoot")]
    pub coins_root: Vec<u8>,
    #[avro(rename = "contractsRoot")]
    pub contracts_root: Vec<u8>,
    #[avro(rename = "messagesRoot")]
    pub messages_root: Vec<u8>,
    #[avro(rename = "transactionsRoot")]
    pub transactions_root: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, AvroSchema)]
#[serde(rename_all = "camelCase")]
pub struct PoAConsensus {
    pub signature: Option<Vec<u8>>,
}

#[derive(
    Debug, Clone, PartialEq, Default, Serialize, Deserialize, AvroSchema,
)]
#[serde(rename_all = "camelCase")]
pub struct AvroBlock {
    pub height: Option<i64>,
    pub time: Option<i64>,
    #[avro(rename = "transactionsCount")]
    pub transactions_count: Option<i64>,
    #[avro(rename = "transactionsRoot")]
    pub transactions_root: Option<Vec<u8>>,
    #[avro(rename = "applicationHash")]
    pub application_hash: Option<Vec<u8>>,
    #[avro(rename = "consensusParametersVersion")]
    pub consensus_parameters_version: Option<i64>,
    #[avro(rename = "daHeight")]
    pub da_height: Option<i64>,
    #[avro(rename = "eventInboxRoot")]
    pub event_inbox_root: Option<Vec<u8>>,
    pub id: Option<Vec<u8>>,
    #[avro(rename = "messageOutboxRoot")]
    pub message_outbox_root: Option<Vec<u8>>,
    #[avro(rename = "messageReceiptCount")]
    pub message_receipt_count: Option<i64>,
    #[avro(rename = "prevRoot")]
    pub prev_root: Option<Vec<u8>>,
    #[avro(rename = "stateTransitionBytecodeVersion")]
    pub state_transition_bytecode_version: Option<i64>,
    pub version: Option<String>,
    #[avro(rename = "consensusType")]
    pub consensus_type: Option<String>,
    #[avro(rename = "genesisData")]
    pub genesis_data: Option<Genesis>,
    #[avro(rename = "poaConsensusDataSignature")]
    pub poa_consensus_data_signature: Option<Vec<u8>>,
    pub producer: Option<Vec<u8>>,
    pub transactions: Vec<AvroTransaction>,
}

impl AvroBlock {
    pub fn new(block: Block, transactions: Vec<AvroTransaction>) -> Self {
        let (consensus_type, genesis_data, poa_data) = match block.consensus {
            Consensus::Genesis(genesis) => (
                Some("Genesis".to_string()),
                Some(Genesis {
                    chain_config_hash: genesis.chain_config_hash.0.to_vec(),
                    coins_root: genesis.coins_root.0.to_vec(),
                    contracts_root: genesis.contracts_root.0.to_vec(),
                    messages_root: genesis.messages_root.0.to_vec(),
                    transactions_root: genesis.transactions_root.0.to_vec(),
                }),
                None,
            ),
            Consensus::PoAConsensus(poa) => (
                Some("PoAConsensus".to_string()),
                None,
                Some(PoAConsensus {
                    signature: Some(poa.signature.0.to_vec()),
                }),
            ),
        };

        Self {
            height: Some(block.height.0 as i64),
            time: Some(block.header.time.0.to_unix()),
            transactions_count: Some(block.transaction_ids.len() as i64),
            transactions_root: Some(block.header.transactions_root.0.to_vec()),
            application_hash: Some(block.header.application_hash.0.to_vec()),
            consensus_parameters_version: Some(
                block.header.consensus_parameters_version.0 as i64,
            ),
            da_height: Some(block.header.da_height.0 as i64),
            event_inbox_root: Some(block.header.event_inbox_root.0.to_vec()),
            id: Some(block.id.0.to_vec()),
            message_outbox_root: Some(
                block.header.message_outbox_root.0.to_vec(),
            ),
            message_receipt_count: Some(
                block.header.message_receipt_count.0 as i64,
            ),
            prev_root: Some(block.header.prev_root.0.to_vec()),
            state_transition_bytecode_version: Some(
                block.header.state_transition_bytecode_version.0 as i64,
            ),
            version: Some(block.version.to_string()),
            consensus_type,
            genesis_data,
            poa_consensus_data_signature: poa_data
                .as_ref()
                .and_then(|p| p.signature.clone()),
            producer: Some(block.producer.0.to_vec()),
            transactions,
        }
    }
}

#[cfg(test)]
mod tests {
    use fuel_streams_domains::blocks::{Block, BlockVersion, Consensus};
    use fuel_streams_types::{
        Address,
        BlockHeader,
        BlockHeaderVersion,
        BlockHeight,
        BlockId,
        BlockTime,
        Bytes32,
        DaBlockHeight,
        Signature,
        WrappedU32,
    };
    use pretty_assertions::assert_eq;

    use super::*;
    use crate::helpers::{write_schema_files, AvroParser, TestBlockMetadata};

    // Helper function for block serialization testing
    fn test_block_serialization(parser: AvroParser, avro_block: AvroBlock) {
        // Test JSON serialization/deserialization
        let ser = serde_json::to_vec(&avro_block).unwrap();
        let deser = serde_json::from_slice::<AvroBlock>(&ser).unwrap();
        assert_eq!(avro_block, deser);

        // Test Avro serialization/deserialization
        let mut avro_writer = parser.writer_with_schema::<AvroBlock>().unwrap();
        let serialized = avro_writer.serialize(&avro_block).unwrap();
        let deserialized = parser
            .reader_with_schema::<AvroBlock>()
            .unwrap()
            .deserialize(&serialized)
            .unwrap();

        assert_eq!(deserialized.len(), 1);
        assert_eq!(avro_block, deserialized[0]);
    }

    fn create_test_block_header() -> BlockHeader {
        let metadata = TestBlockMetadata::new();
        let unix_timestamp = metadata.block_time;

        BlockHeader {
            application_hash: Bytes32::default(),
            consensus_parameters_version: WrappedU32(0),
            da_height: DaBlockHeight::random(),
            event_inbox_root: Bytes32::default(),
            id: BlockId::default(),
            height: BlockHeight(metadata.block_height as u32),
            message_outbox_root: Bytes32::default(),
            message_receipt_count: WrappedU32(0),
            prev_root: Bytes32::default(),
            state_transition_bytecode_version: WrappedU32(0),
            time: BlockTime::from_unix(unix_timestamp),
            transactions_count: 0,
            transactions_root: Bytes32::default(),
            version: BlockHeaderVersion::V1,
        }
    }

    fn create_test_genesis() -> Consensus {
        Consensus::Genesis(fuel_streams_domains::blocks::Genesis {
            chain_config_hash: Bytes32::default(),
            coins_root: Bytes32::default(),
            contracts_root: Bytes32::default(),
            messages_root: Bytes32::default(),
            transactions_root: Bytes32::default(),
        })
    }

    fn create_test_block() -> Block {
        let metadata = TestBlockMetadata::new();

        Block {
            consensus: create_test_genesis(),
            header: create_test_block_header(),
            height: BlockHeight(metadata.block_height as u32),
            id: BlockId::default(),
            transaction_ids: vec![],
            version: BlockVersion::V1,
            producer: Address::random(),
        }
    }

    #[test]
    fn test_avro_block() {
        let parser = AvroParser::default();
        let block = create_test_block();

        // Create AvroBlock
        let avro_block = AvroBlock::new(block.clone(), vec![]);

        test_block_serialization(parser, avro_block);
    }

    #[test]
    fn test_avro_block_with_poa_consensus() {
        let parser = AvroParser::default();
        let mut block = create_test_block();

        // Change consensus to PoA
        block.consensus = Consensus::PoAConsensus(
            fuel_streams_domains::blocks::PoAConsensus {
                signature: Signature::random(),
            },
        );

        // Create AvroBlock
        let avro_block = AvroBlock::new(block.clone(), vec![]);

        test_block_serialization(parser, avro_block);
    }

    #[test]
    fn test_avro_block_with_custom_metadata() {
        let parser = AvroParser::default();

        // Create custom metadata
        let metadata = TestBlockMetadata::with_values(
            100,
            2000,
            vec![4, 5, 6],
            "V1".to_string(),
            vec![7, 8, 9],
        );

        // Create a block with custom values
        let mut block = create_test_block();
        block.height = BlockHeight(metadata.block_height as u32);
        block.header.height = BlockHeight(metadata.block_height as u32);
        block.header.time = BlockTime::from_unix(metadata.block_time);
        block.producer = Address::random();

        // Create AvroBlock
        let avro_block = AvroBlock::new(block, vec![]);

        test_block_serialization(parser, avro_block);
    }

    #[tokio::test]
    async fn write_block_schema() {
        let schemas = [
            ("block.json", AvroBlock::get_schema()),
            ("genesis.json", Genesis::get_schema()),
            ("poa_consensus.json", PoAConsensus::get_schema()),
        ];

        write_schema_files(&schemas).await;
    }
}
