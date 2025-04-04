use apache_avro::AvroSchema;
use fuel_streams_domains::receipts::Receipt;
use fuel_streams_types::{
    FuelCoreScriptExecutionResult,
    FuelCoreWord,
    ScriptExecutionResult,
};
use serde::{Deserialize, Serialize};

#[derive(
    Debug, Clone, PartialEq, Default, Serialize, Deserialize, AvroSchema,
)]
#[serde(rename_all = "camelCase")]
pub struct AvroReceipt {
    #[avro(rename = "blockTime")]
    pub block_time: Option<i64>,
    #[avro(rename = "blockHeight")]
    pub block_height: Option<i64>,
    #[avro(rename = "blockVersion")]
    pub block_version: Option<String>,
    #[avro(rename = "blockProducer")]
    pub block_producer: Option<Vec<u8>>,
    #[avro(rename = "transactionId")]
    pub transaction_id: Option<Vec<u8>>,
    pub amount: Option<i64>,
    #[avro(rename = "assetId")]
    pub asset_id: Option<Vec<u8>>,
    #[avro(rename = "contractId")]
    pub contract_id: Option<Vec<u8>>,
    pub data: Option<String>,
    pub digest: Option<Vec<u8>>,
    pub gas: Option<i64>,
    #[avro(rename = "gasUsed")]
    pub gas_used: Option<i64>,
    pub id: Option<String>,
    pub is: Option<i64>,
    pub len: Option<i64>,
    pub nonce: Option<Vec<u8>>,
    pub param1: Option<i64>,
    pub param2: Option<i64>,
    pub ptr: Option<i64>,
    pub ra: Option<i64>,
    pub rb: Option<i64>,
    pub rc: Option<i64>,
    pub rd: Option<i64>,
    #[avro(rename = "reasonReason")]
    pub reason_reason: Option<u8>,
    #[avro(rename = "reasonInstruction")]
    pub reason_instruction: Option<i64>,
    #[avro(rename = "receiptType")]
    pub receipt_type: Option<String>,
    pub recipient: Option<Vec<u8>>,
    pub result: Option<i64>,
    pub sender: Option<Vec<u8>>,
    #[avro(rename = "subId")]
    pub sub_id: Option<Vec<u8>>,
    pub to: Option<Vec<u8>>,
    #[avro(rename = "toAddress")]
    pub to_address: Option<String>,
    pub val: Option<i64>,
}

#[derive(Debug, Clone)]
pub struct ReceiptMetadata {
    pub block_time: Option<i64>,
    pub block_height: Option<i64>,
    pub block_version: Option<String>,
    pub block_producer: Option<Vec<u8>>,
    pub transaction_id: Option<Vec<u8>>,
}

impl ReceiptMetadata {
    pub fn from_test_metadata(
        metadata: &crate::helpers::TestBlockMetadata,
        tx_id: Vec<u8>,
    ) -> Self {
        Self {
            block_time: Some(metadata.block_time),
            block_height: Some(metadata.block_height),
            block_version: Some(metadata.block_version.clone()),
            block_producer: Some(metadata.block_producer.clone()),
            transaction_id: Some(tx_id),
        }
    }
}

impl AvroReceipt {
    pub fn new(receipt: &Receipt, metadata: &ReceiptMetadata) -> Self {
        // Block-related fields
        let block_time = metadata.block_time;
        let block_height = metadata.block_height;
        let block_version = metadata.block_version.clone();
        let block_producer = metadata.block_producer.clone();
        let transaction_id = metadata.transaction_id.clone();

        // Receipt type-specific fields
        let amount = match receipt {
            Receipt::Call(r) => Some(r.amount.0 as i64),
            Receipt::Transfer(r) => Some(r.amount.0 as i64),
            Receipt::TransferOut(r) => Some(r.amount.0 as i64),
            Receipt::MessageOut(r) => Some(r.amount.0 as i64),
            _ => None,
        };

        let asset_id = match receipt {
            Receipt::Call(r) => Some(r.asset_id.0.to_vec()),
            Receipt::Transfer(r) => Some(r.asset_id.0.to_vec()),
            Receipt::TransferOut(r) => Some(r.asset_id.0.to_vec()),
            _ => None,
        };

        let contract_id = match receipt {
            Receipt::Call(r) => Some(r.id.0.to_vec()),
            Receipt::Return(r) => Some(r.id.0.to_vec()),
            Receipt::ReturnData(r) => Some(r.id.0.to_vec()),
            Receipt::Panic(r) => Some(r.id.0.to_vec()),
            Receipt::Revert(r) => Some(r.id.0.to_vec()),
            Receipt::Log(r) => Some(r.id.0.to_vec()),
            Receipt::LogData(r) => Some(r.id.0.to_vec()),
            Receipt::Transfer(r) => Some(r.id.0.to_vec()),
            Receipt::TransferOut(r) => Some(r.id.0.to_vec()),
            Receipt::Mint(r) => Some(r.contract_id.0.to_vec()),
            Receipt::Burn(r) => Some(r.contract_id.0.to_vec()),
            _ => None,
        };

        let data = match receipt {
            Receipt::ReturnData(r) => r.data.as_ref().map(|d| d.to_string()),
            Receipt::LogData(r) => r.data.as_ref().map(|d| d.to_string()),
            Receipt::MessageOut(r) => r.data.as_ref().map(|d| d.to_string()),
            _ => None,
        };

        let digest = match receipt {
            Receipt::ReturnData(r) => Some(r.digest.0.to_vec()),
            Receipt::LogData(r) => Some(r.digest.0.to_vec()),
            Receipt::MessageOut(r) => Some(r.digest.0.to_vec()),
            _ => None,
        };

        let gas = match receipt {
            Receipt::Call(r) => Some(r.gas.0 as i64),
            _ => None,
        };

        let gas_used = match receipt {
            Receipt::ScriptResult(r) => Some(r.gas_used.0 as i64),
            _ => None,
        };

        let id = match receipt {
            Receipt::Call(r) => Some(r.id.to_string()),
            Receipt::Return(r) => Some(r.id.to_string()),
            Receipt::ReturnData(r) => Some(r.id.to_string()),
            Receipt::Panic(r) => Some(r.id.to_string()),
            Receipt::Revert(r) => Some(r.id.to_string()),
            Receipt::Log(r) => Some(r.id.to_string()),
            Receipt::LogData(r) => Some(r.id.to_string()),
            Receipt::Transfer(r) => Some(r.id.to_string()),
            Receipt::TransferOut(r) => Some(r.id.to_string()),
            _ => None,
        };

        let is = match receipt {
            Receipt::Call(r) => Some(r.is.0 as i64),
            Receipt::Return(r) => Some(r.is.0 as i64),
            Receipt::ReturnData(r) => Some(r.is.0 as i64),
            Receipt::Panic(r) => Some(r.is.0 as i64),
            Receipt::Revert(r) => Some(r.is.0 as i64),
            Receipt::Log(r) => Some(r.is.0 as i64),
            Receipt::LogData(r) => Some(r.is.0 as i64),
            Receipt::Transfer(r) => Some(r.is.0 as i64),
            Receipt::TransferOut(r) => Some(r.is.0 as i64),
            Receipt::Mint(r) => Some(r.is.0 as i64),
            Receipt::Burn(r) => Some(r.is.0 as i64),
            _ => None,
        };

        let len = match receipt {
            Receipt::ReturnData(r) => Some(r.len.0 as i64),
            Receipt::LogData(r) => Some(r.len.0 as i64),
            Receipt::MessageOut(r) => Some(r.len.0 as i64),
            _ => None,
        };

        let nonce = match receipt {
            Receipt::MessageOut(r) => Some(r.nonce.0.to_vec()),
            _ => None,
        };

        let param1 = match receipt {
            Receipt::Call(r) => Some(r.param1.0 as i64),
            _ => None,
        };

        let param2 = match receipt {
            Receipt::Call(r) => Some(r.param2.0 as i64),
            _ => None,
        };

        let ptr = match receipt {
            Receipt::ReturnData(r) => Some(r.ptr.0 as i64),
            Receipt::LogData(r) => Some(r.ptr.0 as i64),
            _ => None,
        };

        let ra = match receipt {
            Receipt::Revert(r) => Some(r.ra.0 as i64),
            Receipt::Log(r) => Some(r.ra.0 as i64),
            Receipt::LogData(r) => Some(r.ra.0 as i64),
            _ => None,
        };

        let rb = match receipt {
            Receipt::Log(r) => Some(r.rb.0 as i64),
            Receipt::LogData(r) => Some(r.rb.0 as i64),
            _ => None,
        };

        let rc = match receipt {
            Receipt::Log(r) => Some(r.rc.0 as i64),
            _ => None,
        };

        let rd = match receipt {
            Receipt::Log(r) => Some(r.rd.0 as i64),
            _ => None,
        };

        let (reason_reason, reason_instruction) = match receipt {
            Receipt::Panic(r) => {
                (Some(r.reason.reason), Some(r.reason.instruction as i64))
            }
            _ => (None, None),
        };

        let receipt_type = Some(
            match receipt {
                Receipt::Call(_) => "call",
                Receipt::Return(_) => "return",
                Receipt::ReturnData(_) => "return_data",
                Receipt::Panic(_) => "panic",
                Receipt::Revert(_) => "revert",
                Receipt::Log(_) => "log",
                Receipt::LogData(_) => "log_data",
                Receipt::Transfer(_) => "transfer",
                Receipt::TransferOut(_) => "transfer_out",
                Receipt::ScriptResult(_) => "script_result",
                Receipt::MessageOut(_) => "message_out",
                Receipt::Mint(_) => "mint",
                Receipt::Burn(_) => "burn",
            }
            .to_string(),
        );

        let recipient = match receipt {
            Receipt::MessageOut(r) => Some(r.recipient.0.to_vec()),
            _ => None,
        };

        let result = match receipt {
            Receipt::ScriptResult(r) => {
                let result = match r.result {
                    ScriptExecutionResult::Success => {
                        FuelCoreScriptExecutionResult::Success
                    }
                    ScriptExecutionResult::Revert => {
                        FuelCoreScriptExecutionResult::Revert
                    }
                    ScriptExecutionResult::Panic => {
                        FuelCoreScriptExecutionResult::Panic
                    }
                    ScriptExecutionResult::GenericFailure(value) => {
                        FuelCoreScriptExecutionResult::GenericFailure(value)
                    }
                    _ => unreachable!(),
                };
                let result = FuelCoreWord::from(result);
                Some(result as i64)
            }
            _ => None,
        };

        let sender = match receipt {
            Receipt::MessageOut(r) => Some(r.sender.0.to_vec()),
            _ => None,
        };

        let sub_id = match receipt {
            Receipt::Mint(r) => Some(r.sub_id.0.to_vec()),
            Receipt::Burn(r) => Some(r.sub_id.0.to_vec()),
            _ => None,
        };

        let to = match receipt {
            Receipt::Call(r) => Some(r.to.0.to_vec()),
            Receipt::Transfer(r) => Some(r.to.0.to_vec()),
            _ => None,
        };

        let to_address = match receipt {
            Receipt::TransferOut(r) => Some(r.to.to_string()),
            _ => None,
        };

        let val = match receipt {
            Receipt::Return(r) => Some(r.val.0 as i64),
            Receipt::Mint(r) => Some(r.val.0 as i64),
            Receipt::Burn(r) => Some(r.val.0 as i64),
            _ => None,
        };

        Self {
            block_time,
            block_height,
            block_version,
            block_producer,
            transaction_id,
            amount,
            asset_id,
            contract_id,
            data,
            digest,
            gas,
            gas_used,
            id,
            is,
            len,
            nonce,
            param1,
            param2,
            ptr,
            ra,
            rb,
            rc,
            rd,
            reason_reason,
            reason_instruction,
            receipt_type,
            recipient,
            result,
            sender,
            sub_id,
            to,
            to_address,
            val,
        }
    }
}

#[cfg(test)]
mod tests {
    use apache_avro::AvroSchema;
    use fuel_streams_domains::receipts::types::MockReceipt;
    use pretty_assertions::assert_eq;

    use super::*;
    use crate::helpers::{write_schema_files, AvroParser, TestBlockMetadata};

    // Helper function to reduce code duplication in tests
    fn test_receipt_serialization(
        parser: AvroParser,
        avro_receipt: AvroReceipt,
    ) {
        // Test JSON serialization/deserialization
        let ser = serde_json::to_vec(&avro_receipt).unwrap();
        let deser = serde_json::from_slice::<AvroReceipt>(&ser).unwrap();
        assert_eq!(avro_receipt, deser);

        // Test Avro serialization/deserialization
        let mut avro_writer =
            parser.writer_with_schema::<AvroReceipt>().unwrap();
        let serialized = avro_writer.serialize(&avro_receipt).unwrap();
        let deserialized = parser
            .reader_with_schema::<AvroReceipt>()
            .unwrap()
            .deserialize(&serialized)
            .unwrap();

        assert_eq!(deserialized.len(), 1);
        assert_eq!(avro_receipt, deserialized[0]);
    }

    fn create_receipt_metadata() -> ReceiptMetadata {
        let metadata = TestBlockMetadata::new();
        let transaction_id = vec![4, 5, 6];

        ReceiptMetadata::from_test_metadata(&metadata, transaction_id)
    }

    #[test]
    fn test_avro_receipt_call() {
        let parser = AvroParser::default();
        let receipt = MockReceipt::call();
        let metadata = create_receipt_metadata();

        let avro_receipt = AvroReceipt::new(&receipt, &metadata);

        test_receipt_serialization(parser, avro_receipt);
    }

    #[test]
    fn test_avro_receipt_return() {
        let parser = AvroParser::default();
        let receipt = MockReceipt::return_receipt();
        let metadata = create_receipt_metadata();
        let avro_receipt = AvroReceipt::new(&receipt, &metadata);

        test_receipt_serialization(parser, avro_receipt);
    }

    #[test]
    fn test_avro_receipt_return_data() {
        let parser = AvroParser::default();
        let receipt = MockReceipt::return_data();
        let metadata = create_receipt_metadata();
        let avro_receipt = AvroReceipt::new(&receipt, &metadata);

        test_receipt_serialization(parser, avro_receipt);
    }

    #[test]
    fn test_avro_receipt_panic() {
        let parser = AvroParser::default();
        let receipt = MockReceipt::panic();
        let metadata = create_receipt_metadata();
        let avro_receipt = AvroReceipt::new(&receipt, &metadata);

        test_receipt_serialization(parser, avro_receipt);
    }

    #[test]
    fn test_avro_receipt_revert() {
        let parser = AvroParser::default();
        let receipt = MockReceipt::revert();
        let metadata = create_receipt_metadata();
        let avro_receipt = AvroReceipt::new(&receipt, &metadata);

        test_receipt_serialization(parser, avro_receipt);
    }

    #[test]
    fn test_avro_receipt_log() {
        let parser = AvroParser::default();
        let receipt = MockReceipt::log();
        let metadata = create_receipt_metadata();
        let avro_receipt = AvroReceipt::new(&receipt, &metadata);

        test_receipt_serialization(parser, avro_receipt);
    }

    #[test]
    fn test_avro_receipt_log_data() {
        let parser = AvroParser::default();
        let receipt = MockReceipt::log_data();
        let metadata = create_receipt_metadata();
        let avro_receipt = AvroReceipt::new(&receipt, &metadata);

        test_receipt_serialization(parser, avro_receipt);
    }

    #[test]
    fn test_avro_receipt_transfer() {
        let parser = AvroParser::default();
        let receipt = MockReceipt::transfer();
        let metadata = create_receipt_metadata();
        let avro_receipt = AvroReceipt::new(&receipt, &metadata);

        test_receipt_serialization(parser, avro_receipt);
    }

    #[test]
    fn test_avro_receipt_transfer_out() {
        let parser = AvroParser::default();
        let receipt = MockReceipt::transfer_out();
        let metadata = create_receipt_metadata();
        let avro_receipt = AvroReceipt::new(&receipt, &metadata);

        test_receipt_serialization(parser, avro_receipt);
    }

    #[test]
    fn test_avro_receipt_script_result() {
        let parser = AvroParser::default();
        let receipt = MockReceipt::script_result();
        let metadata = create_receipt_metadata();
        let avro_receipt = AvroReceipt::new(&receipt, &metadata);

        test_receipt_serialization(parser, avro_receipt);
    }

    #[test]
    fn test_avro_receipt_message_out() {
        let parser = AvroParser::default();
        let receipt = MockReceipt::message_out();
        let metadata = create_receipt_metadata();
        let avro_receipt = AvroReceipt::new(&receipt, &metadata);

        test_receipt_serialization(parser, avro_receipt);
    }

    #[test]
    fn test_avro_receipt_mint() {
        let parser = AvroParser::default();
        let receipt = MockReceipt::mint();
        let metadata = create_receipt_metadata();
        let avro_receipt = AvroReceipt::new(&receipt, &metadata);

        test_receipt_serialization(parser, avro_receipt);
    }

    #[test]
    fn test_avro_receipt_burn() {
        let parser = AvroParser::default();
        let receipt = MockReceipt::burn();
        let metadata = create_receipt_metadata();
        let avro_receipt = AvroReceipt::new(&receipt, &metadata);

        test_receipt_serialization(parser, avro_receipt);
    }

    #[test]
    fn test_avro_receipts_all() {
        let parser = AvroParser::default();
        let receipts = MockReceipt::all();
        let test_metadata = TestBlockMetadata::new();
        let metadata =
            ReceiptMetadata::from_test_metadata(&test_metadata, vec![4, 5, 6]);

        for receipt in receipts {
            let avro_receipt = AvroReceipt::new(&receipt, &metadata);
            test_receipt_serialization(parser.clone(), avro_receipt);
        }
    }

    #[test]
    fn test_avro_receipt_with_custom_metadata() {
        let parser = AvroParser::default();
        let receipt = MockReceipt::call();

        // Create custom metadata
        let test_metadata = TestBlockMetadata::with_values(
            100,
            2000,
            vec![4, 5, 6],
            "2.0".to_string(),
            vec![7, 8, 9],
        );
        let metadata =
            ReceiptMetadata::from_test_metadata(&test_metadata, vec![
                10, 11, 12,
            ]);

        let avro_receipt = AvroReceipt::new(&receipt, &metadata);
        test_receipt_serialization(parser, avro_receipt);
    }

    #[tokio::test]
    async fn write_receipt_schema() {
        let schemas = [("receipt.json", AvroReceipt::get_schema())];
        write_schema_files(&schemas).await;
    }
}
