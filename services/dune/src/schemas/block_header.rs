use apache_avro::AvroSchema;
use fuel_streams_types::{BlockHeader, UnixTimestamp};
use serde::{Deserialize, Serialize};

use crate::helpers::AvroBytes;

#[derive(
    Debug, Clone, PartialEq, Default, Serialize, Deserialize, AvroSchema,
)]
#[serde(rename_all = "camelCase")]
pub struct AvroBlockHeader {
    #[avro(rename = "applicationHash")]
    pub application_hash: Option<AvroBytes>,
    #[avro(rename = "consensusParametersVersion")]
    pub consensus_parameters_version: Option<i64>,
    #[avro(rename = "daHeight")]
    pub da_height: Option<i64>,
    #[avro(rename = "eventInboxRoot")]
    pub event_inbox_root: Option<AvroBytes>,
    pub id: Option<AvroBytes>,
    pub height: Option<i64>,
    #[avro(rename = "messageOutboxRoot")]
    pub message_outbox_root: Option<AvroBytes>,
    #[avro(rename = "messageReceiptCount")]
    pub message_receipt_count: Option<i64>,
    #[avro(rename = "prevRoot")]
    pub prev_root: Option<AvroBytes>,
    #[avro(rename = "stateTransitionBytecodeVersion")]
    pub state_transition_bytecode_version: Option<i64>,
    pub time: Option<i64>,
    #[avro(rename = "transactionsCount")]
    pub transactions_count: Option<i64>,
    #[avro(rename = "transactionsRoot")]
    pub transactions_root: Option<AvroBytes>,
    pub version: Option<String>,
    #[avro(rename = "rescuedData")]
    pub _rescued_data: Option<String>,
    #[avro(rename = "updatedAt")]
    pub updated_at: Option<i64>,
    #[avro(rename = "ingestedAt")]
    pub ingested_at: Option<i64>,
    #[avro(rename = "sourceFilePath")]
    pub _source_file_path: Option<String>,
}

impl AvroBlockHeader {
    pub fn new(
        block_header: BlockHeader,
        rescued_data: Option<String>,
        updated_at: Option<UnixTimestamp>,
        ingested_at: Option<UnixTimestamp>,
        source_file_path: Option<String>,
    ) -> Self {
        Self {
            application_hash: Some(
                block_header.application_hash.clone().into(),
            ),
            consensus_parameters_version: Some(
                block_header.consensus_parameters_version.0 as i64,
            ),
            da_height: Some(block_header.da_height.0 as i64),
            event_inbox_root: Some(
                block_header.event_inbox_root.clone().into(),
            ),
            id: Some(block_header.id.clone().into()),
            height: Some(block_header.height.0 as i64),
            message_outbox_root: Some(
                block_header.message_outbox_root.clone().into(),
            ),
            message_receipt_count: Some(
                block_header.message_receipt_count.0 as i64,
            ),
            prev_root: Some(block_header.prev_root.clone().into()),
            state_transition_bytecode_version: Some(
                block_header.state_transition_bytecode_version.0 as i64,
            ),
            time: Some(block_header.time.0.to_unix()),
            transactions_count: Some(block_header.transactions_count as i64),
            transactions_root: Some(
                block_header.transactions_root.clone().into(),
            ),
            version: Some(block_header.version.to_string()),
            _rescued_data: rescued_data,
            updated_at: updated_at.map(|t| *t.0 as i64),
            ingested_at: ingested_at.map(|t| *t.0 as i64),
            _source_file_path: source_file_path,
        }
    }
}

#[cfg(test)]
mod tests {
    use fuel_streams_types::{Amount, BlockHeader, UnixTimestamp};
    use pretty_assertions::assert_eq;

    use super::*;
    use crate::helpers::{write_schema_files, AvroParser, TestBlockMetadata};

    fn test_block_header_serialization(
        parser: AvroParser,
        avro_header: AvroBlockHeader,
    ) {
        let ser = serde_json::to_vec(&avro_header).unwrap();
        let deser = serde_json::from_slice::<AvroBlockHeader>(&ser).unwrap();
        assert_eq!(avro_header, deser);

        let mut avro_writer =
            parser.writer_with_schema::<AvroBlockHeader>().unwrap();
        avro_writer.append(&avro_header).unwrap();
        let serialized = avro_writer.into_inner().unwrap();
        let deserialized = parser
            .reader_with_schema::<AvroBlockHeader>()
            .unwrap()
            .deserialize(&serialized)
            .unwrap();

        assert_eq!(deserialized.len(), 1);
        assert_eq!(avro_header, deserialized[0]);
    }

    #[test]
    fn test_avro_block_header() {
        let parser = AvroParser::default();
        let block_header = BlockHeader::default();

        let metadata = TestBlockMetadata::new();
        let unix_timestamp = metadata.block_time;

        let avro_block_header = AvroBlockHeader::new(
            block_header,
            Some("test_rescue".to_string()),
            Some(UnixTimestamp(Amount(unix_timestamp as u64))),
            Some(UnixTimestamp(Amount(unix_timestamp as u64))),
            Some("test/path".to_string()),
        );

        test_block_header_serialization(parser, avro_block_header);
    }

    #[test]
    fn test_avro_block_header_custom() {
        let parser = AvroParser::default();
        let block_header = BlockHeader::default();
        let metadata = TestBlockMetadata::default();
        let unix_timestamp = metadata.block_time;
        let avro_block_header = AvroBlockHeader::new(
            block_header,
            Some("custom_rescue".to_string()),
            Some(UnixTimestamp(Amount(unix_timestamp as u64))),
            Some(UnixTimestamp(Amount(unix_timestamp as u64))),
            Some("custom/path".to_string()),
        );

        test_block_header_serialization(parser, avro_block_header);
    }

    #[tokio::test]
    async fn write_block_header_schema() {
        let schemas = [("block_header.json", AvroBlockHeader::get_schema())];

        write_schema_files(&schemas).await;
    }
}
