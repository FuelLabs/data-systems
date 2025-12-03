use apache_avro::AvroSchema;
use fuel_streams_domains::{
    blocks::Block,
    transactions::Transaction,
};
use fuel_streams_types::{
    FuelCoreUpgradePurpose,
    Policies as DomainPolicies,
    StorageSlot,
    TxPointer as CoreTxPointer,
    UpgradePurpose as DomainUpgradePurpose,
};
use serde::{
    Deserialize,
    Serialize,
};

use super::{
    InputContract,
    OutputContract,
};
use crate::helpers::AvroBytes;

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize, AvroSchema)]
#[serde(rename_all = "camelCase")]
pub struct TxPointer {
    #[avro(rename = "blockHeight")]
    pub block_height: Option<i64>,
    #[avro(rename = "txIndex")]
    pub tx_index: Option<i32>,
}

impl From<&CoreTxPointer> for TxPointer {
    fn from(tx_pointer: &CoreTxPointer) -> Self {
        Self {
            block_height: Some(tx_pointer.block_height.into_inner() as i64),
            tx_index: Some(tx_pointer.tx_index as i32),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize, AvroSchema)]
#[serde(rename_all = "camelCase")]
pub struct Policies {
    pub maturity: Option<i64>,
    #[avro(rename = "maxFee")]
    pub max_fee: Option<i64>,
    pub tip: Option<i64>,
    #[avro(rename = "witnessLimit")]
    pub witness_limit: Option<i64>,
}

impl Policies {
    pub fn new(policies: &DomainPolicies) -> Self {
        Self {
            maturity: policies.maturity.map(|m| m.into()),
            max_fee: policies.max_fee.map(|f| f.into()),
            tip: policies.tip.map(|t| t.into()),
            witness_limit: policies.witness_limit.map(|w| w.into()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize, AvroSchema)]
#[serde(rename_all = "camelCase")]
pub struct UpgradePurpose {
    #[avro(rename = "purposeType")]
    pub purpose_type: Option<String>,
    #[avro(rename = "witnessIndex")]
    pub witness_index: Option<i64>,
    pub checksum: Option<AvroBytes>,
    pub root: Option<AvroBytes>,
}

impl UpgradePurpose {
    pub fn new(purpose: &DomainUpgradePurpose) -> Self {
        match purpose.0 {
            FuelCoreUpgradePurpose::ConsensusParameters {
                witness_index,
                checksum,
            } => Self {
                purpose_type: Some("ConsensusParameters".to_string()),
                witness_index: Some(witness_index as i64),
                checksum: Some(checksum.to_vec().into()),
                root: None,
            },
            FuelCoreUpgradePurpose::StateTransition { root } => Self {
                purpose_type: Some("StateTransition".to_string()),
                witness_index: None,
                checksum: None,
                root: Some(root.to_vec().into()),
            },
        }
    }
}

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize, AvroSchema)]
#[serde(rename_all = "camelCase")]
pub struct AvroStorageSlot {
    pub key: AvroBytes,
    pub value: AvroBytes,
}

impl From<&StorageSlot> for AvroStorageSlot {
    fn from(slot: &StorageSlot) -> Self {
        Self {
            key: slot.key.as_ref().to_vec().into(),
            value: slot.value.as_ref().to_vec().into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize, AvroSchema)]
#[serde(rename_all = "camelCase")]
pub struct AvroTransaction {
    #[avro(rename = "blockHeight")]
    pub block_height: Option<i64>,
    #[avro(rename = "blockTime")]
    pub block_time: Option<i64>,
    #[avro(rename = "blockId")]
    pub block_id: Option<AvroBytes>,
    #[avro(rename = "blockVersion")]
    pub block_version: Option<String>,
    #[avro(rename = "blockProducer")]
    pub block_producer: Option<AvroBytes>,
    pub status: Option<String>,
    pub id: Option<AvroBytes>,
    #[avro(rename = "type")]
    pub r#type: Option<String>,
    #[avro(rename = "txPointer")]
    pub tx_pointer: Option<TxPointer>,
    #[avro(rename = "bytecodeRoot")]
    pub bytecode_root: Option<AvroBytes>,
    #[avro(rename = "bytecodeWitnessIndex")]
    pub bytecode_witness_index: Option<i64>,
    #[avro(rename = "blobId")]
    pub blob_id: Option<AvroBytes>,
    pub maturity: Option<i64>,
    #[avro(rename = "mintAmount")]
    pub mint_amount: Option<i64>,
    #[avro(rename = "mintAssetId")]
    pub mint_asset_id: Option<AvroBytes>,
    #[avro(rename = "mintGasPrice")]
    pub mint_gas_price: Option<i64>,
    #[avro(rename = "receiptsRoot")]
    pub receipts_root: Option<AvroBytes>,
    pub salt: Option<AvroBytes>,
    #[avro(rename = "scriptGasLimit")]
    pub script_gas_limit: Option<i64>,
    #[avro(rename = "subsectionIndex")]
    pub subsection_index: Option<i64>,
    #[avro(rename = "subsectionsNumber")]
    pub subsections_number: Option<i64>,
    #[avro(rename = "inputAssetIds")]
    pub input_asset_ids: Option<Vec<AvroBytes>>,
    #[avro(rename = "proofSet")]
    pub proof_set: Option<Vec<AvroBytes>>,
    #[avro(rename = "inputContract")]
    pub input_contract: Option<InputContract>,
    #[avro(rename = "outputContract")]
    pub output_contract: Option<OutputContract>,
    pub policies: Option<Policies>,
    #[avro(rename = "rawPayload")]
    pub raw_payload: Option<AvroBytes>,
    pub script: Option<AvroBytes>,
    #[avro(rename = "scriptData")]
    pub script_data: Option<AvroBytes>,
    #[avro(rename = "storageSlots")]
    pub storage_slots: Option<Vec<AvroStorageSlot>>,
    #[avro(rename = "upgradePurpose")]
    pub upgrade_purpose: Option<UpgradePurpose>,
    pub witnesses: Option<Vec<AvroBytes>>,
    #[avro(rename = "scriptLength")]
    pub script_length: Option<i64>,
    #[avro(rename = "scriptDataLength")]
    pub script_data_length: Option<i64>,
    #[avro(rename = "storageSlotsCount")]
    pub storage_slots_count: Option<i64>,
    #[avro(rename = "proofSetCount")]
    pub proof_set_count: Option<i64>,
    #[avro(rename = "witnessesCount")]
    pub witnesses_count: Option<i64>,
    #[avro(rename = "inputsCount")]
    pub inputs_count: Option<i64>,
    #[avro(rename = "outputsCount")]
    pub outputs_count: Option<i64>,
    #[avro(rename = "isCreate")]
    pub is_create: Option<bool>,
    #[avro(rename = "isMint")]
    pub is_mint: Option<bool>,
    #[avro(rename = "isScript")]
    pub is_script: Option<bool>,
    #[avro(rename = "isUpgrade")]
    pub is_upgrade: Option<bool>,
    #[avro(rename = "isUpload")]
    pub is_upload: Option<bool>,
    #[avro(rename = "isBlob")]
    pub is_blob: Option<bool>,
}

impl AvroTransaction {
    pub fn new(
        transaction: &Transaction,
        block_height: Option<i64>,
        block_time: Option<i64>,
        block_id: Option<AvroBytes>,
        block_version: Option<String>,
        block_producer: Option<AvroBytes>,
    ) -> Self {
        let status = Some(transaction.status.to_string());
        let storage_slots = transaction
            .storage_slots
            .as_ref()
            .map(|slots| slots.iter().map(AvroStorageSlot::from).collect());
        Self {
            block_height,
            block_time,
            block_id,
            block_version,
            block_producer,
            status,
            id: Some(transaction.id.as_ref().to_vec().into()),
            r#type: Some(transaction.r#type.to_string()),
            tx_pointer: transaction.tx_pointer.as_ref().map(TxPointer::from),
            bytecode_root: transaction
                .bytecode_root
                .as_ref()
                .map(|br| br.as_ref().to_vec().into()),
            bytecode_witness_index: transaction.bytecode_witness_index.map(Into::into),
            blob_id: transaction
                .blob_id
                .as_ref()
                .map(|bid| bid.as_ref().to_vec().into()),
            maturity: transaction.maturity.map(Into::into),
            mint_amount: transaction
                .mint_amount
                .as_ref()
                .map(|amount| amount.as_ref().to_owned() as i64),
            mint_asset_id: transaction
                .mint_asset_id
                .as_ref()
                .map(|id| id.as_ref().to_vec().into()),
            mint_gas_price: transaction
                .mint_gas_price
                .as_ref()
                .map(|price| price.as_ref().to_owned() as i64),
            receipts_root: transaction
                .receipts_root
                .as_ref()
                .map(|root| root.as_ref().to_vec().into()),
            salt: transaction
                .salt
                .as_ref()
                .map(|s| s.as_ref().to_vec().into()),
            script_gas_limit: transaction
                .script_gas_limit
                .as_ref()
                .map(|limit| limit.as_ref().to_owned() as i64),
            subsection_index: transaction.subsection_index.map(Into::into),
            subsections_number: transaction.subsections_number.map(Into::into),
            input_asset_ids: transaction
                .input_asset_ids
                .as_ref()
                .map(|ids| ids.iter().map(|id| id.as_ref().to_vec().into()).collect()),
            proof_set: transaction.proof_set.as_ref().map(|proofs| {
                proofs.iter().map(|p| p.as_ref().to_vec().into()).collect()
            }),
            input_contract: transaction.input_contract.as_ref().map(InputContract::new),
            output_contract: transaction
                .output_contract
                .as_ref()
                .map(OutputContract::new),
            policies: transaction.policies.as_ref().map(Policies::new),
            raw_payload: Some(transaction.raw_payload.as_ref().as_ref().to_vec().into()),
            script: transaction
                .script
                .as_ref()
                .map(|s| s.as_ref().as_ref().to_vec().into()),
            script_data: transaction
                .script_data
                .as_ref()
                .map(|s| s.as_ref().as_ref().to_vec().into()),
            storage_slots,
            upgrade_purpose: transaction
                .upgrade_purpose
                .as_ref()
                .map(UpgradePurpose::new),
            witnesses: transaction.witnesses.as_ref().map(|w| {
                w.iter()
                    .map(|witness| witness.as_ref().as_ref().to_vec().into())
                    .collect()
            }),
            script_length: transaction.script_length.map(Into::into),
            script_data_length: transaction.script_data_length.map(Into::into),
            storage_slots_count: Some(transaction.storage_slots_count.into()),
            proof_set_count: Some(transaction.proof_set_count.into()),
            witnesses_count: Some(transaction.witnesses_count.into()),
            inputs_count: Some(transaction.inputs_count.into()),
            outputs_count: Some(transaction.outputs_count.into()),
            is_create: Some(transaction.is_create),
            is_mint: Some(transaction.is_mint),
            is_script: Some(transaction.is_script),
            is_upgrade: Some(transaction.is_upgrade),
            is_upload: Some(transaction.is_upload),
            is_blob: Some(transaction.is_blob),
        }
    }
}

impl From<(&Block, &Transaction)> for AvroTransaction {
    fn from((block, transaction): (&Block, &Transaction)) -> Self {
        AvroTransaction::new(
            transaction,
            Some(block.height.into()),
            Some(block.header.get_timestamp_utc().timestamp()),
            Some(block.id.as_ref().to_vec().into()),
            Some(block.version.to_string()),
            Some(block.producer.as_ref().to_vec().into()),
        )
    }
}

#[cfg(test)]
mod tests {
    use fuel_streams_domains::{
        inputs::types::MockInput,
        outputs::types::MockOutput,
        transactions::MockTransaction,
    };
    use pretty_assertions::assert_eq;

    use super::*;
    use crate::helpers::{
        AvroParser,
        TestBlockMetadata,
        write_schema_files,
    };

    fn test_transaction_serialization(parser: AvroParser, avro_tx: AvroTransaction) {
        let ser = serde_json::to_vec(&avro_tx).unwrap();
        let deser = serde_json::from_slice::<AvroTransaction>(&ser).unwrap();
        assert_eq!(avro_tx, deser);

        let mut avro_writer = parser.writer_with_schema::<AvroTransaction>().unwrap();
        avro_writer.append(&avro_tx).unwrap();
        let serialized = avro_writer.into_inner().unwrap();
        let deserialized = parser
            .reader_with_schema::<AvroTransaction>()
            .unwrap()
            .deserialize(&serialized)
            .unwrap();

        assert_eq!(deserialized.len(), 1);
        assert_eq!(avro_tx, deserialized[0]);
    }

    #[test]
    fn test_avro_transaction_script() {
        let parser = AvroParser::default();
        let tx = MockTransaction::script(vec![], vec![], vec![]);
        let metadata = TestBlockMetadata::new();
        let (block_height, block_time, block_id, block_version, block_producer) =
            metadata.as_options();

        let avro_tx = AvroTransaction::new(
            &tx,
            block_height,
            block_time,
            block_id,
            block_version,
            block_producer,
        );

        test_transaction_serialization(parser, avro_tx);
    }

    #[test]
    fn test_avro_transaction_create() {
        let parser = AvroParser::default();
        let tx = MockTransaction::create(vec![], vec![], vec![]);
        let metadata = TestBlockMetadata::new();
        let (block_height, block_time, block_id, block_version, block_producer) =
            metadata.as_options();

        let avro_tx = AvroTransaction::new(
            &tx,
            block_height,
            block_time,
            block_id,
            block_version,
            block_producer,
        );

        test_transaction_serialization(parser, avro_tx);
    }

    #[test]
    fn test_avro_transaction_mint() {
        let parser = AvroParser::default();
        let tx = MockTransaction::mint(vec![], vec![], vec![]);
        let metadata = TestBlockMetadata::new();
        let (block_height, block_time, block_id, block_version, block_producer) =
            metadata.as_options();

        let avro_tx = AvroTransaction::new(
            &tx,
            block_height,
            block_time,
            block_id,
            block_version,
            block_producer,
        );

        test_transaction_serialization(parser, avro_tx);
    }

    #[test]
    fn test_avro_transaction_upgrade() {
        let parser = AvroParser::default();
        let tx = MockTransaction::upgrade(vec![], vec![], vec![]);
        let metadata = TestBlockMetadata::new();
        let (block_height, block_time, block_id, block_version, block_producer) =
            metadata.as_options();

        let avro_tx = AvroTransaction::new(
            &tx,
            block_height,
            block_time,
            block_id,
            block_version,
            block_producer,
        );

        test_transaction_serialization(parser, avro_tx);
    }

    #[test]
    fn test_avro_transaction_upload() {
        let parser = AvroParser::default();
        let tx = MockTransaction::upload(vec![], vec![], vec![]);
        let metadata = TestBlockMetadata::new();
        let (block_height, block_time, block_id, block_version, block_producer) =
            metadata.as_options();

        let avro_tx = AvroTransaction::new(
            &tx,
            block_height,
            block_time,
            block_id,
            block_version,
            block_producer,
        );

        test_transaction_serialization(parser, avro_tx);
    }

    #[test]
    fn test_avro_transaction_blob() {
        let parser = AvroParser::default();
        let tx = MockTransaction::blob(vec![], vec![], vec![]);
        let metadata = TestBlockMetadata::new();
        let (block_height, block_time, block_id, block_version, block_producer) =
            metadata.as_options();

        let avro_tx = AvroTransaction::new(
            &tx,
            block_height,
            block_time,
            block_id,
            block_version,
            block_producer,
        );

        test_transaction_serialization(parser, avro_tx);
    }

    #[test]
    fn test_avro_transaction_with_inputs_outputs() {
        let parser = AvroParser::default();
        let inputs = vec![
            MockInput::contract(),
            MockInput::coin_signed(None),
            MockInput::message_coin_signed(),
        ];
        let outputs = vec![
            MockOutput::coin(1000),
            MockOutput::contract(),
            MockOutput::contract_created(),
        ];
        let tx = MockTransaction::script(inputs, outputs, vec![]);
        let metadata = TestBlockMetadata::new();
        let (block_height, block_time, block_id, block_version, block_producer) =
            metadata.as_options();

        let avro_tx = AvroTransaction::new(
            &tx,
            block_height,
            block_time,
            block_id,
            block_version,
            block_producer,
        );

        test_transaction_serialization(parser, avro_tx);
    }

    #[test]
    fn test_avro_transactions_all() {
        let parser = AvroParser::default();
        let transactions = MockTransaction::all();
        let metadata = TestBlockMetadata::new();
        let (block_height, block_time, block_id, block_version, block_producer) =
            metadata.as_options();

        for tx in transactions {
            let avro_tx = AvroTransaction::new(
                &tx,
                block_height,
                block_time,
                block_id.clone(),
                block_version.clone(),
                block_producer.clone(),
            );
            test_transaction_serialization(parser.clone(), avro_tx);
        }
    }

    #[tokio::test]
    async fn write_transaction_schemas() {
        let schemas = [
            ("transaction.json", AvroTransaction::get_schema()),
            ("tx_pointer.json", TxPointer::get_schema()),
            ("policies.json", Policies::get_schema()),
            ("upgrade_purpose.json", UpgradePurpose::get_schema()),
            ("storage_slot.json", AvroStorageSlot::get_schema()),
        ];

        write_schema_files(&schemas).await;
    }
}
