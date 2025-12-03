use apache_avro::AvroSchema;
use fuel_streams_domains::blocks::{
    Block,
    Consensus,
};
use serde::{
    Deserialize,
    Serialize,
};

use crate::helpers::AvroBytes;

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize, AvroSchema)]
#[serde(rename_all = "camelCase")]
pub struct Genesis {
    #[avro(rename = "chainConfigHash")]
    pub chain_config_hash: AvroBytes,
    #[avro(rename = "coinsRoot")]
    pub coins_root: AvroBytes,
    #[avro(rename = "contractsRoot")]
    pub contracts_root: AvroBytes,
    #[avro(rename = "messagesRoot")]
    pub messages_root: AvroBytes,
    #[avro(rename = "transactionsRoot")]
    pub transactions_root: AvroBytes,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, AvroSchema)]
#[serde(rename_all = "camelCase")]
pub struct PoAConsensus {
    pub signature: Option<AvroBytes>,
}

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize, AvroSchema)]
#[serde(rename_all = "camelCase")]
pub struct AvroBlock {
    pub height: Option<i64>,
    pub time: Option<i64>,
    #[avro(rename = "transactionsCount")]
    pub transactions_count: Option<i64>,
    #[avro(rename = "transactionsRoot")]
    pub transactions_root: Option<AvroBytes>,
    #[avro(rename = "applicationHash")]
    pub application_hash: Option<AvroBytes>,
    #[avro(rename = "consensusParametersVersion")]
    pub consensus_parameters_version: Option<i64>,
    #[avro(rename = "daHeight")]
    pub da_height: Option<i64>,
    #[avro(rename = "eventInboxRoot")]
    pub event_inbox_root: Option<AvroBytes>,
    pub id: Option<AvroBytes>,
    #[avro(rename = "messageOutboxRoot")]
    pub message_outbox_root: Option<AvroBytes>,
    #[avro(rename = "messageReceiptCount")]
    pub message_receipt_count: Option<i64>,
    #[avro(rename = "prevRoot")]
    pub prev_root: Option<AvroBytes>,
    #[avro(rename = "stateTransitionBytecodeVersion")]
    pub state_transition_bytecode_version: Option<i64>,
    pub version: Option<String>,
    #[avro(rename = "consensusType")]
    pub consensus_type: Option<String>,
    #[avro(rename = "poaConsensusDataSignature")]
    pub poa_consensus_data_signature: Option<AvroBytes>,
    pub producer: Option<AvroBytes>,
}

impl AvroBlock {
    pub fn new(block: &Block) -> Self {
        let (consensus_type, _, poa_data) = match &block.consensus {
            Consensus::Genesis(genesis) => (
                Some("Genesis".to_string()),
                Some(Genesis {
                    chain_config_hash: genesis.chain_config_hash.clone().into(),
                    coins_root: genesis.coins_root.clone().into(),
                    contracts_root: genesis.contracts_root.clone().into(),
                    messages_root: genesis.messages_root.clone().into(),
                    transactions_root: genesis.transactions_root.clone().into(),
                }),
                None,
            ),
            Consensus::PoAConsensus(poa) => (
                Some("PoAConsensus".to_string()),
                None,
                Some(PoAConsensus {
                    signature: Some(poa.signature.clone().into()),
                }),
            ),
        };

        Self {
            height: Some(block.height.0 as i64),
            time: Some(block.header.time.0.to_unix()),
            transactions_count: Some(block.transaction_count),
            transactions_root: Some(block.header.transactions_root.clone().into()),
            application_hash: Some(block.header.application_hash.clone().into()),
            consensus_parameters_version: Some(
                block.header.consensus_parameters_version.0 as i64,
            ),
            da_height: Some(block.header.da_height.0 as i64),
            event_inbox_root: Some(block.header.event_inbox_root.clone().into()),
            id: Some(block.id.clone().into()),
            message_outbox_root: Some(block.header.message_outbox_root.clone().into()),
            message_receipt_count: Some(block.header.message_receipt_count.0 as i64),
            prev_root: Some(block.header.prev_root.clone().into()),
            state_transition_bytecode_version: Some(
                block.header.state_transition_bytecode_version.0 as i64,
            ),
            version: Some(block.version.to_string()),
            consensus_type,
            poa_consensus_data_signature: poa_data
                .as_ref()
                .and_then(|p| p.signature.clone()),
            producer: Some(block.producer.clone().into()),
        }
    }
}

#[cfg(test)]
mod tests {
    use fuel_streams_domains::{
        blocks::{
            Block,
            Consensus,
        },
        mocks::MockBlock,
    };
    use fuel_streams_types::{
        Address,
        BlockHeight,
        BlockTime,
        Signature,
    };
    use pretty_assertions::assert_eq;

    use super::*;
    use crate::helpers::{
        AvroParser,
        AvroParserError,
        TestBlockMetadata,
        write_schema_files,
    };

    fn test_block_serialization(block: AvroBlock) -> Result<(), AvroParserError> {
        let parser = AvroParser::default();
        let mut avro_writer = parser.writer_with_schema::<AvroBlock>()?;
        avro_writer.append(&block)?;

        let serialized = avro_writer.into_inner()?;
        let deserialized = parser
            .reader_with_schema::<AvroBlock>()?
            .deserialize(&serialized)?;
        assert_eq!(deserialized.len(), 1);
        assert_eq!(deserialized[0], block);
        Ok(())
    }

    fn create_test_block() -> Block {
        MockBlock::random()
    }

    #[test]
    fn test_avro_block() -> anyhow::Result<()> {
        let block = create_test_block();
        let avro_block = AvroBlock::new(&block);
        test_block_serialization(avro_block)?;
        Ok(())
    }

    #[test]
    fn test_avro_block_with_poa_consensus() -> anyhow::Result<()> {
        let mut block = create_test_block();

        block.consensus =
            Consensus::PoAConsensus(fuel_streams_domains::blocks::PoAConsensus {
                signature: Signature::random(),
            });

        let avro_block = AvroBlock::new(&block);
        test_block_serialization(avro_block)?;
        Ok(())
    }

    #[test]
    fn test_avro_block_with_custom_metadata() -> anyhow::Result<()> {
        let metadata = TestBlockMetadata::default();
        let mut block = create_test_block();
        block.height = BlockHeight(metadata.block_height as u32);
        block.header.height = BlockHeight(metadata.block_height as u32);
        block.header.time = BlockTime::from_unix(metadata.block_time);
        block.producer = Address::random();
        let avro_block = AvroBlock::new(&block);
        test_block_serialization(avro_block)?;
        Ok(())
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
