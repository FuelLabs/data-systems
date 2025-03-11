use fuel_core_types::fuel_tx;
use fuel_streams_types::{fuel_core::*, primitives::*};
pub use fuel_streams_types::{TransactionStatus, TransactionType};
use serde::{Deserialize, Serialize};

use crate::{inputs::types::*, outputs::types::*, receipts::types::*};

#[derive(
    Debug, Default, Clone, PartialEq, Serialize, Deserialize, utoipa::ToSchema,
)]
pub struct StorageSlot {
    pub key: HexData,
    pub value: HexData,
}

impl From<FuelCoreStorageSlot> for StorageSlot {
    fn from(slot: FuelCoreStorageSlot) -> Self {
        Self::from(&slot)
    }
}

impl From<&FuelCoreStorageSlot> for StorageSlot {
    fn from(slot: &FuelCoreStorageSlot) -> Self {
        Self {
            key: HexData(slot.key().as_slice().into()),
            value: HexData(slot.value().as_slice().into()),
        }
    }
}

#[derive(
    Debug, Clone, Copy, Serialize, Deserialize, Eq, PartialEq, Default, Hash,
)]
pub struct PolicyWrapper(pub FuelCorePolicies);

impl utoipa::ToSchema for PolicyWrapper {
    fn name() -> std::borrow::Cow<'static, str> {
        std::borrow::Cow::Borrowed("FuelCorePolicies")
    }
}

impl utoipa::PartialSchema for PolicyWrapper {
    fn schema() -> utoipa::openapi::RefOr<utoipa::openapi::schema::Schema> {
        utoipa::openapi::schema::ObjectBuilder::new()
            .schema_type(utoipa::openapi::schema::Type::Array)
            .title(Some("FuelCorePolicies"))
            .description(Some("Array of u64 policy values used by the VM"))
            .property(
                "values",
                utoipa::openapi::schema::ObjectBuilder::new()
                    .schema_type(utoipa::openapi::schema::Type::Integer)
                    .format(Some(
                        utoipa::openapi::schema::SchemaFormat::KnownFormat(
                            utoipa::openapi::KnownFormat::Int64,
                        ),
                    ))
                    .build(),
            )
            .examples([Some(serde_json::json!([0, 0, 0, 0, 0]))])
            .build()
            .into()
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Eq, PartialEq, Hash)]
pub struct FuelCoreUpgradePurposeWrapper(pub FuelCoreUpgradePurpose);

impl From<FuelCoreUpgradePurpose> for FuelCoreUpgradePurposeWrapper {
    fn from(purpose: FuelCoreUpgradePurpose) -> Self {
        FuelCoreUpgradePurposeWrapper(purpose)
    }
}

impl From<FuelCoreUpgradePurposeWrapper> for FuelCoreUpgradePurpose {
    fn from(wrapper: FuelCoreUpgradePurposeWrapper) -> Self {
        wrapper.0
    }
}

impl utoipa::ToSchema for FuelCoreUpgradePurposeWrapper {
    fn name() -> std::borrow::Cow<'static, str> {
        std::borrow::Cow::Borrowed("FuelCoreUpgradePurpose")
    }
}

impl utoipa::PartialSchema for FuelCoreUpgradePurposeWrapper {
    fn schema() -> utoipa::openapi::RefOr<utoipa::openapi::schema::Schema> {
        // Create Object builders first
        let consensus_params_obj = utoipa::openapi::schema::ObjectBuilder::new()
            .schema_type(utoipa::openapi::schema::Type::Object)
            .title(Some("ConsensusParameters"))
            // ... other properties
            .build();

        let state_transition_obj = utoipa::openapi::schema::ObjectBuilder::new()
            .schema_type(utoipa::openapi::schema::Type::Object)
            .title(Some("StateTransition"))
            // ... other properties
            .build();

        // Convert Objects to Schemas
        let consensus_params =
            utoipa::openapi::schema::Schema::Object(consensus_params_obj);
        let state_transition =
            utoipa::openapi::schema::Schema::Object(state_transition_obj);

        // Create a oneOf schema with both variants
        let mut one_of = utoipa::openapi::schema::OneOf::new();

        // Now we can add Schemas to the items
        one_of
            .items
            .push(utoipa::openapi::RefOr::T(consensus_params));
        one_of
            .items
            .push(utoipa::openapi::RefOr::T(state_transition));

        // Create the oneOf schema and return it
        let schema = utoipa::openapi::schema::Schema::OneOf(one_of);

        // Return the Schema
        utoipa::openapi::RefOr::T(schema)
    }
}

#[derive(
    Debug, Default, Clone, PartialEq, Serialize, Deserialize, utoipa::ToSchema,
)]
#[serde(rename_all = "camelCase")]
pub struct Transaction {
    pub id: TxId,
    #[serde(rename = "type")]
    pub tx_type: TransactionType,
    pub bytecode_root: Option<Bytes32>,
    pub bytecode_witness_index: Option<u16>,
    pub blob_id: Option<BlobId>,
    pub input_asset_ids: Option<Vec<AssetId>>,
    pub input_contract: Option<InputContract>,
    pub input_contracts: Option<Vec<ContractId>>,
    pub inputs: Vec<Input>,
    pub output_contract: Option<OutputContract>,
    pub outputs: Vec<Output>,
    pub is_create: bool,
    pub is_mint: bool,
    pub is_script: bool,
    pub is_upgrade: bool,
    pub is_upload: bool,
    pub maturity: Option<u32>,
    pub mint_amount: Option<Amount>,
    pub mint_asset_id: Option<AssetId>,
    pub mint_gas_price: Option<Amount>,
    pub policies: Option<PolicyWrapper>,
    pub proof_set: Vec<Bytes32>,
    pub raw_payload: HexData,
    pub receipts_root: Option<Bytes32>,
    pub salt: Option<Salt>,
    pub script: Option<HexData>,
    pub script_data: Option<HexData>,
    pub script_gas_limit: Option<GasAmount>,
    pub status: TransactionStatus,
    pub storage_slots: Vec<StorageSlot>,
    pub subsection_index: Option<u16>,
    pub subsections_number: Option<u16>,
    pub tx_pointer: Option<TxPointer>,
    pub upgrade_purpose: Option<FuelCoreUpgradePurposeWrapper>,
    pub witnesses: Vec<HexData>,
    pub receipts: Vec<Receipt>,
}

impl Transaction {
    pub fn new(
        id: &Bytes32,
        transaction: &FuelCoreTransaction,
        status: &TransactionStatus,
        base_asset_id: &FuelCoreAssetId,
        receipts: &[FuelCoreReceipt],
    ) -> Self {
        let bytecode_root = {
            use fuel_core_types::fuel_tx::field::BytecodeRoot;
            match transaction {
                FuelCoreTransaction::Upload(tx) => {
                    Some((*tx.bytecode_root()).into())
                }
                _ => None,
            }
        };

        let bytecode_witness_index = {
            use fuel_core_types::fuel_tx::field::BytecodeWitnessIndex;
            match transaction {
                FuelCoreTransaction::Upload(tx) => {
                    Some(*tx.bytecode_witness_index())
                }
                _ => None,
            }
        };

        let blob_id = {
            use fuel_core_types::fuel_tx::field::ChargeableBody;
            match transaction {
                FuelCoreTransaction::Blob(blob) => Some(blob.body().id.into()),
                _ => None,
            }
        };

        let input_asset_ids = {
            use fuel_core_types::fuel_tx::Executable;

            match transaction {
                FuelCoreTransaction::Script(tx) => Some(
                    tx.input_asset_ids(base_asset_id)
                        .map(|c| AssetId::from(*c))
                        .collect(),
                ),
                FuelCoreTransaction::Create(tx) => Some(
                    tx.input_asset_ids(base_asset_id)
                        .map(|c| AssetId::from(*c))
                        .collect(),
                ),
                FuelCoreTransaction::Mint(_) => None,
                FuelCoreTransaction::Upgrade(tx) => Some(
                    tx.input_asset_ids(base_asset_id)
                        .map(|c| AssetId::from(*c))
                        .collect(),
                ),
                FuelCoreTransaction::Upload(tx) => Some(
                    tx.input_asset_ids(base_asset_id)
                        .map(|c| AssetId::from(*c))
                        .collect(),
                ),
                FuelCoreTransaction::Blob(tx) => Some(
                    tx.input_asset_ids(base_asset_id)
                        .map(|c| AssetId::from(*c))
                        .collect(),
                ),
            }
        };

        let input_contract = {
            use fuel_core_types::fuel_tx::field::InputContract;
            match transaction {
                FuelCoreTransaction::Mint(mint) => {
                    Some(mint.input_contract().into())
                }
                _ => None,
            }
        };

        let input_contracts = {
            match transaction {
                FuelCoreTransaction::Mint(_) => None,
                tx => {
                    let mut inputs: Vec<_> = tx
                        .inputs()
                        .iter()
                        .filter_map(|input| match input {
                            fuel_tx::Input::Contract(contract) => {
                                Some(contract.contract_id)
                            }
                            _ => None,
                        })
                        .collect();
                    inputs.sort();
                    inputs.dedup();
                    Some(inputs.into_iter().map(|id| (*id).into()).collect())
                }
            }
        };

        let output_contract = {
            use fuel_core_types::fuel_tx::field::OutputContract;
            match transaction {
                FuelCoreTransaction::Mint(mint) => {
                    Some(mint.output_contract().into())
                }
                _ => None,
            }
        };

        let maturity = {
            use fuel_core_types::fuel_tx::field::Maturity;
            match transaction {
                FuelCoreTransaction::Script(tx) => Some(*tx.maturity()),
                FuelCoreTransaction::Create(tx) => Some(*tx.maturity()),
                FuelCoreTransaction::Mint(_) => None,
                FuelCoreTransaction::Upgrade(tx) => Some(*tx.maturity()),
                FuelCoreTransaction::Upload(tx) => Some(*tx.maturity()),
                FuelCoreTransaction::Blob(tx) => Some(*tx.maturity()),
            }
        };

        let mint_gas_price = {
            use fuel_core_types::fuel_tx::field::MintGasPrice;
            match transaction {
                FuelCoreTransaction::Mint(mint) => Some(*mint.gas_price()),
                _ => None,
            }
        };

        let mint_amount = {
            use fuel_core_types::fuel_tx::field::MintAmount;
            match transaction {
                FuelCoreTransaction::Mint(mint) => Some(*mint.mint_amount()),
                _ => None,
            }
        };

        let mint_asset_id = {
            use fuel_core_types::fuel_tx::field::MintAssetId;
            match transaction {
                FuelCoreTransaction::Mint(mint) => {
                    Some((*mint.mint_asset_id()).into())
                }
                _ => None,
            }
        };

        let policies = {
            use fuel_core_types::fuel_tx::field::Policies;
            match transaction {
                FuelCoreTransaction::Script(tx) => Some(*tx.policies()),
                FuelCoreTransaction::Create(tx) => Some(*tx.policies()),
                FuelCoreTransaction::Mint(_) => None,
                FuelCoreTransaction::Upgrade(tx) => Some(*tx.policies()),
                FuelCoreTransaction::Upload(tx) => Some(*tx.policies()),
                FuelCoreTransaction::Blob(tx) => Some(*tx.policies()),
            }
        };

        let proof_set = {
            use fuel_core_types::fuel_tx::field::ProofSet;
            match transaction {
                FuelCoreTransaction::Upload(tx) => {
                    tx.proof_set().iter().map(|proof| (*proof).into()).collect()
                }
                _ => vec![],
            }
        };

        let raw_payload = {
            use fuel_core_types::fuel_types::canonical::Serialize;
            HexData(transaction.to_bytes().into())
        };

        let receipts_root = {
            use fuel_core_types::fuel_tx::field::ReceiptsRoot;
            match transaction {
                FuelCoreTransaction::Script(script) => {
                    Some((*script.receipts_root()).into())
                }
                _ => None,
            }
        };

        let salt = {
            use fuel_core_types::fuel_tx::field::Salt;
            match transaction {
                FuelCoreTransaction::Create(create) => {
                    Some((*create.salt()).into())
                }
                _ => None,
            }
        };

        let script = {
            use fuel_core_types::fuel_tx::field::Script;
            match transaction {
                FuelCoreTransaction::Script(script) => {
                    Some(HexData(script.script().clone().into()))
                }
                _ => None,
            }
        };

        let script_data = {
            use fuel_core_types::fuel_tx::field::ScriptData;
            match transaction {
                FuelCoreTransaction::Script(script) => {
                    Some(HexData(script.script_data().clone().into()))
                }
                _ => None,
            }
        };

        let script_gas_limit = {
            use fuel_core_types::fuel_tx::field::ScriptGasLimit;
            match transaction {
                FuelCoreTransaction::Script(script) => {
                    Some(*script.script_gas_limit())
                }
                _ => None,
            }
        };

        let storage_slots = {
            use fuel_core_types::fuel_tx::field::StorageSlots;
            match transaction {
                FuelCoreTransaction::Create(create) => create
                    .storage_slots()
                    .iter()
                    .map(|slot| slot.into())
                    .collect(),
                _ => vec![],
            }
        };

        let subsection_index = {
            use fuel_core_types::fuel_tx::field::SubsectionIndex;
            match transaction {
                FuelCoreTransaction::Upload(tx) => Some(*tx.subsection_index()),
                _ => None,
            }
        };

        let subsections_number = {
            use fuel_core_types::fuel_tx::field::SubsectionsNumber;
            match transaction {
                FuelCoreTransaction::Upload(tx) => {
                    Some(*tx.subsections_number())
                }
                _ => None,
            }
        };

        let tx_pointer = {
            use fuel_core_types::fuel_tx::field::TxPointer;
            match transaction {
                FuelCoreTransaction::Mint(mint) => Some(*mint.tx_pointer()),
                _ => None,
            }
        };

        let upgrade_purpose = {
            use fuel_core_types::fuel_tx::field::UpgradePurpose;
            match transaction {
                FuelCoreTransaction::Upgrade(tx) => Some(*tx.upgrade_purpose()),
                _ => None,
            }
        };

        // hexstring encode should be HexData(data)
        let witnesses = {
            use fuel_core_types::fuel_tx::field::Witnesses;
            match transaction {
                FuelCoreTransaction::Script(tx) => tx
                    .witnesses()
                    .iter()
                    .map(|w| HexData(w.clone().into_inner().into()))
                    .collect(),
                FuelCoreTransaction::Create(tx) => tx
                    .witnesses()
                    .iter()
                    .map(|w| HexData(w.clone().into_inner().into()))
                    .collect(),
                FuelCoreTransaction::Mint(_) => vec![],
                FuelCoreTransaction::Upgrade(tx) => tx
                    .witnesses()
                    .iter()
                    .map(|w| HexData(w.clone().into_inner().into()))
                    .collect(),
                FuelCoreTransaction::Upload(tx) => tx
                    .witnesses()
                    .iter()
                    .map(|w| HexData(w.clone().into_inner().into()))
                    .collect(),
                FuelCoreTransaction::Blob(tx) => tx
                    .witnesses()
                    .iter()
                    .map(|w| HexData(w.clone().into_inner().into()))
                    .collect(),
            }
        };

        Transaction {
            id: id.to_owned().into(),
            tx_type: transaction.into(),
            bytecode_root,
            bytecode_witness_index,
            blob_id,
            input_asset_ids,
            input_contract,
            input_contracts,
            inputs: transaction.inputs().iter().map(Into::into).collect(),
            output_contract,
            outputs: transaction.outputs().iter().map(Into::into).collect(),
            is_create: transaction.is_create(),
            is_mint: transaction.is_mint(),
            is_script: transaction.is_script(),
            is_upgrade: transaction.is_upgrade(),
            is_upload: transaction.is_upload(),
            maturity,
            mint_amount: mint_amount.map(|amount| amount.into()),
            mint_asset_id,
            mint_gas_price: mint_gas_price.map(|amount| amount.into()),
            policies: policies.map(PolicyWrapper),
            proof_set,
            raw_payload,
            receipts_root,
            salt,
            script,
            script_data,
            script_gas_limit: script_gas_limit.map(|amount| amount.into()),
            status: status.to_owned(),
            storage_slots,
            subsection_index,
            subsections_number,
            tx_pointer: Some(tx_pointer.into()),
            upgrade_purpose: upgrade_purpose.map(FuelCoreUpgradePurposeWrapper),
            witnesses,
            receipts: receipts.iter().map(|r| r.to_owned().into()).collect(),
        }
    }
}

pub trait FuelCoreTransactionExt {
    fn inputs(&self) -> &[FuelCoreInput];
    fn outputs(&self) -> &Vec<FuelCoreOutput>;
}

impl FuelCoreTransactionExt for FuelCoreTransaction {
    fn inputs(&self) -> &[FuelCoreInput] {
        match self {
            FuelCoreTransaction::Mint(_) => &[],
            FuelCoreTransaction::Script(tx) => tx.inputs(),
            FuelCoreTransaction::Blob(tx) => tx.inputs(),
            FuelCoreTransaction::Create(tx) => tx.inputs(),
            FuelCoreTransaction::Upload(tx) => tx.inputs(),
            FuelCoreTransaction::Upgrade(tx) => tx.inputs(),
        }
    }

    fn outputs(&self) -> &Vec<FuelCoreOutput> {
        match self {
            FuelCoreTransaction::Mint(_) => {
                static NO_OUTPUTS: Vec<FuelCoreOutput> = Vec::new();
                &NO_OUTPUTS
            }
            FuelCoreTransaction::Script(tx) => tx.outputs(),
            FuelCoreTransaction::Blob(tx) => tx.outputs(),
            FuelCoreTransaction::Create(tx) => tx.outputs(),
            FuelCoreTransaction::Upload(tx) => tx.outputs(),
            FuelCoreTransaction::Upgrade(tx) => tx.outputs(),
        }
    }
}

#[cfg(any(test, feature = "test-helpers"))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MockTransaction;
#[cfg(any(test, feature = "test-helpers"))]
impl MockTransaction {
    fn base_transaction(tx_type: TransactionType) -> Transaction {
        Transaction {
            id: TxId::random(),
            tx_type: tx_type.clone(),
            bytecode_root: None,
            bytecode_witness_index: None,
            blob_id: None,
            input_asset_ids: Some(vec![AssetId::default()]),
            input_contract: None,
            input_contracts: None,
            inputs: vec![MockInput::coin_signed()],
            output_contract: None,
            outputs: vec![MockOutput::coin(100)],
            is_create: tx_type == TransactionType::Create,
            is_mint: tx_type == TransactionType::Mint,
            is_script: tx_type == TransactionType::Script,
            is_upgrade: tx_type == TransactionType::Upgrade,
            is_upload: tx_type == TransactionType::Upload,
            maturity: Some(0),
            mint_amount: None,
            mint_asset_id: None,
            mint_gas_price: None,
            policies: Some(PolicyWrapper::default()),
            proof_set: vec![],
            raw_payload: HexData::default(),
            receipts_root: None,
            salt: None,
            script: None,
            script_data: None,
            script_gas_limit: None,
            status: TransactionStatus::Success,
            storage_slots: vec![],
            subsection_index: None,
            subsections_number: None,
            tx_pointer: Some(TxPointer::default()),
            upgrade_purpose: None,
            witnesses: vec![HexData::default()],
            receipts: vec![MockReceipt::script_result()],
        }
    }

    fn with_script_data(
        mut tx: Transaction,
        script: Vec<u8>,
        script_data: Vec<u8>,
    ) -> Transaction {
        tx.script = Some(HexData(script.into()));
        tx.script_data = Some(HexData(script_data.into()));
        tx.script_gas_limit = Some(1000.into());
        tx
    }

    fn with_contract_data(mut tx: Transaction) -> Transaction {
        tx.output_contract = Some(OutputContract {
            balance_root: Bytes32::default(),
            input_index: 0,
            state_root: Bytes32::default(),
        });
        tx.storage_slots = vec![StorageSlot {
            key: HexData::default(),
            value: HexData::default(),
        }];
        tx
    }

    fn with_mint_data(mut tx: Transaction) -> Transaction {
        tx.input_contract = Some(InputContract {
            balance_root: Bytes32::default(),
            contract_id: Bytes32::default(),
            state_root: Bytes32::default(),
            tx_pointer: TxPointer::default(),
            utxo_id: UtxoId::default(),
        });
        tx.mint_amount = Some(1000.into());
        tx.mint_asset_id = Some(AssetId::default());
        tx.mint_gas_price = Some(100.into());
        tx.tx_pointer = Some(TxPointer::default());
        tx
    }

    pub fn script(
        inputs: Vec<Input>,
        outputs: Vec<Output>,
        receipts: Vec<Receipt>,
    ) -> Transaction {
        let mut tx = Self::with_script_data(
            Self::base_transaction(TransactionType::Script),
            vec![1, 2, 3],
            vec![4, 5, 6],
        );
        tx.inputs = inputs;
        tx.outputs = outputs;
        tx.receipts = receipts;
        tx
    }

    pub fn create(
        inputs: Vec<Input>,
        outputs: Vec<Output>,
        receipts: Vec<Receipt>,
    ) -> Transaction {
        let mut tx = Self::base_transaction(TransactionType::Create);
        tx.salt = Some(Salt::default());
        tx.inputs = inputs;
        tx.outputs = outputs;
        tx.receipts = receipts;
        Self::with_contract_data(tx)
    }

    pub fn mint(
        inputs: Vec<Input>,
        outputs: Vec<Output>,
        receipts: Vec<Receipt>,
    ) -> Transaction {
        let mut tx = Self::base_transaction(TransactionType::Mint);
        tx.inputs = inputs;
        tx.outputs = outputs;
        tx.receipts = receipts;
        Self::with_mint_data(tx)
    }

    pub fn upgrade(
        inputs: Vec<Input>,
        outputs: Vec<Output>,
        receipts: Vec<Receipt>,
    ) -> Transaction {
        let mut tx = Self::base_transaction(TransactionType::Upgrade);
        tx.upgrade_purpose = Some(
            FuelCoreUpgradePurpose::StateTransition {
                root: FuelCoreBytes32::default(),
            }
            .into(),
        );
        tx.inputs = inputs;
        tx.outputs = outputs;
        tx.receipts = receipts;
        tx
    }

    pub fn upload(
        inputs: Vec<Input>,
        outputs: Vec<Output>,
        receipts: Vec<Receipt>,
    ) -> Transaction {
        let mut tx = Self::base_transaction(TransactionType::Upload);
        tx.bytecode_root = Some(Bytes32::default());
        tx.bytecode_witness_index = Some(0);
        tx.proof_set = vec![Bytes32::default()];
        tx.subsection_index = Some(0);
        tx.subsections_number = Some(1);
        tx.inputs = inputs;
        tx.outputs = outputs;
        tx.receipts = receipts;
        tx
    }

    pub fn blob(
        inputs: Vec<Input>,
        outputs: Vec<Output>,
        receipts: Vec<Receipt>,
    ) -> Transaction {
        let mut tx = Self::base_transaction(TransactionType::Blob);
        tx.blob_id = Some(BlobId::default());
        tx.inputs = inputs;
        tx.outputs = outputs;
        tx.receipts = receipts;
        tx
    }

    pub fn all() -> Vec<Transaction> {
        let inputs = MockInput::all();
        let outputs = MockOutput::all();
        let receipts = MockReceipt::all();
        vec![
            Self::script(inputs.clone(), outputs.clone(), receipts.clone()),
            Self::create(inputs.clone(), outputs.clone(), receipts.clone()),
            Self::mint(inputs.clone(), outputs.clone(), receipts.clone()),
            Self::upgrade(inputs.clone(), outputs.clone(), receipts.clone()),
            Self::upload(inputs.clone(), outputs.clone(), receipts.clone()),
            Self::blob(inputs.clone(), outputs.clone(), receipts.clone()),
        ]
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    use serde_json::json;

    use super::*;

    #[test]
    fn test_transaction_serialization_deserialization() {
        // Create a mock transaction
        let original_tx = MockTransaction::script(
            MockInput::all(),
            MockOutput::all(),
            MockReceipt::all(),
        );

        // Serialize to JSON
        let serialized = serde_json::to_string(&original_tx).unwrap();

        // Deserialize back to Transaction
        let deserialized: Transaction =
            serde_json::from_str(&serialized).unwrap();

        // Verify the deserialized transaction matches the original
        assert_eq!(deserialized, original_tx);

        // Verify specific fields are correctly serialized
        let json_value: serde_json::Value =
            serde_json::from_str(&serialized).unwrap();

        // Check transaction type is serialized as lowercase string
        assert_eq!(json_value["type"], json!("script"));

        // Check transaction status is serialized as lowercase string
        assert_eq!(json_value["status"], json!("success"));

        // Test with all transaction types
        for tx in MockTransaction::all() {
            let serialized = serde_json::to_string(&tx).unwrap();
            let deserialized: Transaction =
                serde_json::from_str(&serialized).unwrap();
            assert_eq!(deserialized, tx);

            // Verify type field is correctly serialized as lowercase
            let json_value: serde_json::Value =
                serde_json::from_str(&serialized).unwrap();
            assert_eq!(json_value["type"], json!(tx.tx_type.as_str()));
        }
    }
}
