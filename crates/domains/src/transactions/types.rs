use fuel_core_types::fuel_tx;
use fuel_data_parser::DataEncoder;
use fuel_streams_types::{fuel_core::*, primitives::*};
pub use fuel_streams_types::{TransactionStatus, TransactionType};
use serde::{Deserialize, Serialize};

use crate::{
    infra::record::ToPacket,
    inputs::types::*,
    outputs::types::*,
    receipts::types::*,
};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, utoipa::ToSchema)]
pub struct Transaction {
    pub id: TxId,
    pub r#type: TransactionType,
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
    pub is_blob: bool,
    pub mint_amount: Option<Amount>,
    pub mint_asset_id: Option<AssetId>,
    pub mint_gas_price: Option<Amount>,
    pub proof_set: Option<Vec<Bytes32>>,
    pub raw_payload: HexData,
    pub receipts_root: Option<Bytes32>,
    pub salt: Option<Salt>,
    pub script: Option<HexData>,
    pub script_data: Option<HexData>,
    pub script_gas_limit: Option<GasAmount>,
    pub status: TransactionStatus,
    pub storage_slots: Option<Vec<StorageSlot>>,
    pub subsection_index: Option<u16>,
    pub subsections_number: Option<u16>,
    pub tx_pointer: Option<TxPointer>,
    pub upgrade_purpose: Option<UpgradePurpose>,
    pub witnesses: Option<Vec<HexData>>,
    pub receipts: Vec<Receipt>,

    pub policies: Option<Policies>,
    pub maturity: Option<u32>,
    pub script_length: Option<u32>,
    pub script_data_length: Option<u32>,
    pub storage_slots_count: u32,
    pub proof_set_count: u32,
    pub witnesses_count: u32,
    pub inputs_count: u32,
    pub outputs_count: u32,
}

impl DataEncoder for Transaction {}
impl ToPacket for Transaction {}

impl Transaction {
    pub fn new(
        id: &Bytes32,
        transaction: &FuelCoreTypesTransaction,
        status: &TransactionStatus,
        base_asset_id: &FuelCoreAssetId,
        receipts: &[FuelCoreReceipt],
    ) -> Self {
        Self {
            id: id.to_owned().into(),
            r#type: transaction.into(),
            bytecode_root: Self::get_bytecode_root(transaction),
            bytecode_witness_index: Self::get_bytecode_witness_index(
                transaction,
            ),
            blob_id: Self::get_blob_id(transaction),
            input_asset_ids: Self::get_input_asset_ids(
                transaction,
                base_asset_id,
            ),
            input_contract: Self::get_input_contract(transaction),
            input_contracts: Self::get_input_contracts(transaction),
            inputs: transaction.inputs().iter().map(Into::into).collect(),
            output_contract: Self::get_output_contract(transaction),
            outputs: transaction.outputs().iter().map(Into::into).collect(),
            is_create: transaction.is_create(),
            is_mint: transaction.is_mint(),
            is_script: transaction.is_script(),
            is_upgrade: transaction.is_upgrade(),
            is_upload: transaction.is_upload(),
            is_blob: transaction.is_blob(),
            mint_amount: Self::get_mint_amount(transaction),
            mint_asset_id: Self::get_mint_asset_id(transaction),
            mint_gas_price: Self::get_mint_gas_price(transaction),
            proof_set: Self::get_proof_set(transaction),
            raw_payload: Self::get_raw_payload(transaction),
            receipts_root: Self::get_receipts_root(transaction),
            salt: Self::get_salt(transaction),
            script: Self::get_script(transaction),
            script_data: Self::get_script_data(transaction),
            script_gas_limit: Self::get_script_gas_limit(transaction),
            status: status.to_owned(),
            storage_slots: Self::get_storage_slots(transaction),
            subsection_index: Self::get_subsection_index(transaction),
            subsections_number: Self::get_subsections_number(transaction),
            tx_pointer: Self::get_tx_pointer(transaction),
            upgrade_purpose: Self::get_upgrade_purpose(transaction),
            witnesses: Self::get_witnesses(transaction),
            receipts: receipts.iter().map(|r| r.to_owned().into()).collect(),

            maturity: Self::get_maturity(transaction),
            policies: Self::get_policies(transaction),
            script_length: Self::get_script_length(transaction),
            script_data_length: Self::get_script_data_length(transaction),
            storage_slots_count: Self::get_storage_slots_count(transaction),
            proof_set_count: Self::get_proof_set_count(transaction),
            witnesses_count: Self::get_witnesses_count(transaction),
            inputs_count: Self::get_inputs_count(transaction),
            outputs_count: Self::get_outputs_count(transaction),
        }
    }

    fn get_bytecode_root(
        transaction: &FuelCoreTypesTransaction,
    ) -> Option<Bytes32> {
        use fuel_core_types::fuel_tx::field::BytecodeRoot;
        match transaction {
            FuelCoreTypesTransaction::Upload(tx) => {
                Some((*tx.bytecode_root()).into())
            }
            _ => None,
        }
    }

    fn get_bytecode_witness_index(
        transaction: &FuelCoreTypesTransaction,
    ) -> Option<u16> {
        use fuel_core_types::fuel_tx::field::BytecodeWitnessIndex;
        match transaction {
            FuelCoreTypesTransaction::Upload(tx) => {
                Some(*tx.bytecode_witness_index())
            }
            _ => None,
        }
    }

    fn get_blob_id(transaction: &FuelCoreTypesTransaction) -> Option<BlobId> {
        use fuel_core_types::fuel_tx::field::BlobId;
        match transaction {
            FuelCoreTypesTransaction::Blob(blob) => Some(blob.blob_id().into()),
            _ => None,
        }
    }

    fn get_input_asset_ids(
        transaction: &FuelCoreTypesTransaction,
        base_asset_id: &FuelCoreAssetId,
    ) -> Option<Vec<AssetId>> {
        use fuel_core_types::fuel_tx::Executable;
        match transaction {
            FuelCoreTypesTransaction::Script(tx) => Some(
                tx.input_asset_ids(base_asset_id)
                    .map(|c| AssetId::from(*c))
                    .collect(),
            ),
            FuelCoreTypesTransaction::Create(tx) => Some(
                tx.input_asset_ids(base_asset_id)
                    .map(|c| AssetId::from(*c))
                    .collect(),
            ),
            FuelCoreTypesTransaction::Mint(_) => None,
            FuelCoreTypesTransaction::Upgrade(tx) => Some(
                tx.input_asset_ids(base_asset_id)
                    .map(|c| AssetId::from(*c))
                    .collect(),
            ),
            FuelCoreTypesTransaction::Upload(tx) => Some(
                tx.input_asset_ids(base_asset_id)
                    .map(|c| AssetId::from(*c))
                    .collect(),
            ),
            FuelCoreTypesTransaction::Blob(tx) => Some(
                tx.input_asset_ids(base_asset_id)
                    .map(|c| AssetId::from(*c))
                    .collect(),
            ),
        }
    }

    fn get_input_contract(
        transaction: &FuelCoreTypesTransaction,
    ) -> Option<InputContract> {
        use fuel_core_types::fuel_tx::field::InputContract;
        match transaction {
            FuelCoreTypesTransaction::Mint(mint) => {
                Some(mint.input_contract().into())
            }
            _ => None,
        }
    }

    fn get_input_contracts(
        transaction: &FuelCoreTypesTransaction,
    ) -> Option<Vec<ContractId>> {
        match transaction {
            FuelCoreTypesTransaction::Mint(_) => None,
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
    }

    fn get_output_contract(
        transaction: &FuelCoreTypesTransaction,
    ) -> Option<OutputContract> {
        use fuel_core_types::fuel_tx::field::OutputContract;
        match transaction {
            FuelCoreTypesTransaction::Mint(mint) => {
                Some(mint.output_contract().into())
            }
            _ => None,
        }
    }

    fn get_maturity(transaction: &FuelCoreTypesTransaction) -> Option<u32> {
        use fuel_core_types::fuel_tx::field::Maturity;
        match transaction {
            FuelCoreTypesTransaction::Script(tx) => Some(*tx.maturity()),
            FuelCoreTypesTransaction::Create(tx) => Some(*tx.maturity()),
            FuelCoreTypesTransaction::Mint(_) => None,
            FuelCoreTypesTransaction::Upgrade(tx) => Some(*tx.maturity()),
            FuelCoreTypesTransaction::Upload(tx) => Some(*tx.maturity()),
            FuelCoreTypesTransaction::Blob(tx) => Some(*tx.maturity()),
        }
    }

    fn get_mint_amount(
        transaction: &FuelCoreTypesTransaction,
    ) -> Option<Amount> {
        use fuel_core_types::fuel_tx::field::MintAmount;
        match transaction {
            FuelCoreTypesTransaction::Mint(mint) => {
                Some((*mint.mint_amount()).into())
            }
            _ => None,
        }
    }

    fn get_mint_asset_id(
        transaction: &FuelCoreTypesTransaction,
    ) -> Option<AssetId> {
        use fuel_core_types::fuel_tx::field::MintAssetId;
        match transaction {
            FuelCoreTypesTransaction::Mint(mint) => {
                Some((*mint.mint_asset_id()).into())
            }
            _ => None,
        }
    }

    fn get_mint_gas_price(
        transaction: &FuelCoreTypesTransaction,
    ) -> Option<Amount> {
        use fuel_core_types::fuel_tx::field::MintGasPrice;
        match transaction {
            FuelCoreTypesTransaction::Mint(mint) => {
                Some((*mint.gas_price()).into())
            }
            _ => None,
        }
    }

    fn get_policies(
        transaction: &FuelCoreTypesTransaction,
    ) -> Option<Policies> {
        use fuel_core_types::fuel_tx::field::Policies;
        match transaction {
            FuelCoreTypesTransaction::Script(tx) => {
                Some((*tx.policies()).into())
            }
            FuelCoreTypesTransaction::Create(tx) => {
                Some((*tx.policies()).into())
            }
            FuelCoreTypesTransaction::Mint(_) => None,
            FuelCoreTypesTransaction::Upgrade(tx) => {
                Some((*tx.policies()).into())
            }
            FuelCoreTypesTransaction::Upload(tx) => {
                Some((*tx.policies()).into())
            }
            FuelCoreTypesTransaction::Blob(tx) => Some((*tx.policies()).into()),
        }
    }

    fn get_proof_set(
        transaction: &FuelCoreTypesTransaction,
    ) -> Option<Vec<Bytes32>> {
        use fuel_core_types::fuel_tx::field::ProofSet;
        match transaction {
            FuelCoreTypesTransaction::Upload(tx) => Some(
                tx.proof_set().iter().map(|proof| (*proof).into()).collect(),
            ),
            _ => None,
        }
    }

    fn get_raw_payload(transaction: &FuelCoreTypesTransaction) -> HexData {
        use fuel_core_types::fuel_types::canonical::Serialize;
        HexData(transaction.to_bytes().into())
    }

    fn get_receipts_root(
        transaction: &FuelCoreTypesTransaction,
    ) -> Option<Bytes32> {
        use fuel_core_types::fuel_tx::field::ReceiptsRoot;
        match transaction {
            FuelCoreTypesTransaction::Script(script) => {
                Some((*script.receipts_root()).into())
            }
            _ => None,
        }
    }

    fn get_salt(transaction: &FuelCoreTypesTransaction) -> Option<Salt> {
        use fuel_core_types::fuel_tx::field::Salt;
        match transaction {
            FuelCoreTypesTransaction::Create(create) => {
                Some((*create.salt()).into())
            }
            _ => None,
        }
    }

    fn get_script(transaction: &FuelCoreTypesTransaction) -> Option<HexData> {
        use fuel_core_types::fuel_tx::field::Script;
        match transaction {
            FuelCoreTypesTransaction::Script(script) => {
                Some(HexData(script.script().clone().into()))
            }
            _ => None,
        }
    }

    fn get_script_data(
        transaction: &FuelCoreTypesTransaction,
    ) -> Option<HexData> {
        use fuel_core_types::fuel_tx::field::ScriptData;
        match transaction {
            FuelCoreTypesTransaction::Script(script) => {
                Some(HexData(script.script_data().clone().into()))
            }
            _ => None,
        }
    }

    fn get_script_gas_limit(
        transaction: &FuelCoreTypesTransaction,
    ) -> Option<GasAmount> {
        use fuel_core_types::fuel_tx::field::ScriptGasLimit;
        match transaction {
            FuelCoreTypesTransaction::Script(script) => {
                Some((*script.script_gas_limit()).into())
            }
            _ => None,
        }
    }

    fn get_storage_slots(
        transaction: &FuelCoreTypesTransaction,
    ) -> Option<Vec<StorageSlot>> {
        use fuel_core_types::fuel_tx::field::StorageSlots;
        match transaction {
            FuelCoreTypesTransaction::Create(create) => Some(
                create
                    .storage_slots()
                    .iter()
                    .map(|slot| slot.into())
                    .collect(),
            ),
            _ => None,
        }
    }

    fn get_subsection_index(
        transaction: &FuelCoreTypesTransaction,
    ) -> Option<u16> {
        use fuel_core_types::fuel_tx::field::SubsectionIndex;
        match transaction {
            FuelCoreTypesTransaction::Upload(tx) => {
                Some(*tx.subsection_index())
            }
            _ => None,
        }
    }

    fn get_subsections_number(
        transaction: &FuelCoreTypesTransaction,
    ) -> Option<u16> {
        use fuel_core_types::fuel_tx::field::SubsectionsNumber;
        match transaction {
            FuelCoreTypesTransaction::Upload(tx) => {
                Some(*tx.subsections_number())
            }
            _ => None,
        }
    }

    fn get_tx_pointer(
        transaction: &FuelCoreTypesTransaction,
    ) -> Option<TxPointer> {
        use fuel_core_types::fuel_tx::field::TxPointer;
        match transaction {
            FuelCoreTypesTransaction::Mint(mint) => {
                Some((*mint.tx_pointer()).into())
            }
            _ => None,
        }
    }

    fn get_upgrade_purpose(
        transaction: &FuelCoreTypesTransaction,
    ) -> Option<UpgradePurpose> {
        use fuel_core_types::fuel_tx::field::UpgradePurpose;
        match transaction {
            FuelCoreTypesTransaction::Upgrade(tx) => {
                Some((*tx.upgrade_purpose()).into())
            }
            _ => None,
        }
    }

    fn get_witnesses(
        transaction: &FuelCoreTypesTransaction,
    ) -> Option<Vec<HexData>> {
        use fuel_core_types::fuel_tx::field::Witnesses;
        match transaction {
            FuelCoreTypesTransaction::Script(tx) => Some(
                tx.witnesses()
                    .iter()
                    .map(|w| HexData(w.clone().into_inner().into()))
                    .collect(),
            ),
            FuelCoreTypesTransaction::Create(tx) => Some(
                tx.witnesses()
                    .iter()
                    .map(|w| HexData(w.clone().into_inner().into()))
                    .collect(),
            ),
            FuelCoreTypesTransaction::Mint(_) => None,
            FuelCoreTypesTransaction::Upgrade(tx) => Some(
                tx.witnesses()
                    .iter()
                    .map(|w| HexData(w.clone().into_inner().into()))
                    .collect(),
            ),
            FuelCoreTypesTransaction::Upload(tx) => Some(
                tx.witnesses()
                    .iter()
                    .map(|w| HexData(w.clone().into_inner().into()))
                    .collect(),
            ),
            FuelCoreTypesTransaction::Blob(tx) => Some(
                tx.witnesses()
                    .iter()
                    .map(|w| HexData(w.clone().into_inner().into()))
                    .collect(),
            ),
        }
    }

    fn get_script_length(
        transaction: &FuelCoreTypesTransaction,
    ) -> Option<u32> {
        use fuel_core_types::fuel_tx::field::Script;
        match transaction {
            FuelCoreTypesTransaction::Script(script) => {
                Some(script.script().len() as u32)
            }
            _ => None,
        }
    }

    fn get_script_data_length(
        transaction: &FuelCoreTypesTransaction,
    ) -> Option<u32> {
        use fuel_core_types::fuel_tx::field::ScriptData;
        match transaction {
            FuelCoreTypesTransaction::Script(script) => {
                Some(script.script_data().len() as u32)
            }
            _ => None,
        }
    }

    fn get_storage_slots_count(transaction: &FuelCoreTypesTransaction) -> u32 {
        use fuel_core_types::fuel_tx::field::StorageSlots;
        match transaction {
            FuelCoreTypesTransaction::Create(create) => {
                create.storage_slots().len() as u32
            }
            _ => 0,
        }
    }

    fn get_proof_set_count(transaction: &FuelCoreTypesTransaction) -> u32 {
        use fuel_core_types::fuel_tx::field::ProofSet;
        match transaction {
            FuelCoreTypesTransaction::Upload(tx) => tx.proof_set().len() as u32,
            _ => 0,
        }
    }

    fn get_witnesses_count(transaction: &FuelCoreTypesTransaction) -> u32 {
        use fuel_core_types::fuel_tx::field::Witnesses;
        match transaction {
            FuelCoreTypesTransaction::Script(tx) => tx.witnesses().len() as u32,
            FuelCoreTypesTransaction::Create(tx) => tx.witnesses().len() as u32,
            FuelCoreTypesTransaction::Mint(_) => 0,
            FuelCoreTypesTransaction::Upgrade(tx) => {
                tx.witnesses().len() as u32
            }
            FuelCoreTypesTransaction::Upload(tx) => tx.witnesses().len() as u32,
            FuelCoreTypesTransaction::Blob(tx) => tx.witnesses().len() as u32,
        }
    }

    fn get_inputs_count(transaction: &FuelCoreTypesTransaction) -> u32 {
        transaction.inputs().len() as u32
    }

    fn get_outputs_count(transaction: &FuelCoreTypesTransaction) -> u32 {
        transaction.outputs().len() as u32
    }
}

pub trait FuelCoreTypesTransactionExt {
    fn inputs(&self) -> &[FuelCoreInput];
    fn outputs(&self) -> &Vec<FuelCoreOutput>;
}

impl FuelCoreTypesTransactionExt for FuelCoreTypesTransaction {
    fn inputs(&self) -> &[FuelCoreInput] {
        match self {
            FuelCoreTypesTransaction::Mint(_) => &[],
            FuelCoreTypesTransaction::Script(tx) => tx.inputs(),
            FuelCoreTypesTransaction::Blob(tx) => tx.inputs(),
            FuelCoreTypesTransaction::Create(tx) => tx.inputs(),
            FuelCoreTypesTransaction::Upload(tx) => tx.inputs(),
            FuelCoreTypesTransaction::Upgrade(tx) => tx.inputs(),
        }
    }

    fn outputs(&self) -> &Vec<FuelCoreOutput> {
        match self {
            FuelCoreTypesTransaction::Mint(_) => {
                static NO_OUTPUTS: Vec<FuelCoreOutput> = Vec::new();
                &NO_OUTPUTS
            }
            FuelCoreTypesTransaction::Script(tx) => tx.outputs(),
            FuelCoreTypesTransaction::Blob(tx) => tx.outputs(),
            FuelCoreTypesTransaction::Create(tx) => tx.outputs(),
            FuelCoreTypesTransaction::Upload(tx) => tx.outputs(),
            FuelCoreTypesTransaction::Upgrade(tx) => tx.outputs(),
        }
    }
}

#[cfg(any(test, feature = "test-helpers"))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MockTransaction;

#[cfg(any(test, feature = "test-helpers"))]
impl MockTransaction {
    fn base_transaction(r#type: TransactionType) -> Transaction {
        Transaction {
            id: TxId::random(),
            r#type,
            bytecode_root: None,
            bytecode_witness_index: None,
            blob_id: None,
            input_asset_ids: Some(vec![AssetId::random()]),
            input_contract: None,
            input_contracts: None,
            inputs: vec![MockInput::coin_signed(None)],
            output_contract: None,
            outputs: vec![MockOutput::coin(100)],
            is_create: r#type == TransactionType::Create,
            is_mint: r#type == TransactionType::Mint,
            is_script: r#type == TransactionType::Script,
            is_upgrade: r#type == TransactionType::Upgrade,
            is_upload: r#type == TransactionType::Upload,
            is_blob: r#type == TransactionType::Blob,
            maturity: Some(0),
            mint_amount: None,
            mint_asset_id: None,
            mint_gas_price: None,
            policies: Some(Policies::random()),
            proof_set: None,
            raw_payload: HexData::random(),
            receipts_root: None,
            salt: None,
            script: None,
            script_data: None,
            script_gas_limit: None,
            status: TransactionStatus::Success,
            storage_slots: None,
            subsection_index: None,
            subsections_number: None,
            tx_pointer: Some(TxPointer::random()),
            upgrade_purpose: None,
            witnesses: None,
            receipts: vec![MockReceipt::script_result()],
            // New fields with u32
            script_length: None,
            script_data_length: None,
            storage_slots_count: 0,
            proof_set_count: 0,
            witnesses_count: 1, // One witness by default
            inputs_count: 1,    // One input by default
            outputs_count: 1,   // One output by default
        }
    }

    fn with_script_data(
        mut tx: Transaction,
        script: Vec<u8>,
        script_data: Vec<u8>,
    ) -> Transaction {
        tx.script = Some(HexData(script.clone().into()));
        tx.script_data = Some(HexData(script_data.clone().into()));
        tx.script_gas_limit = Some(1000.into());
        tx.script_length = Some(script.len() as u32);
        tx.script_data_length = Some(script_data.len() as u32);
        tx
    }

    fn with_contract_data(mut tx: Transaction) -> Transaction {
        tx.output_contract = Some(OutputContract {
            balance_root: Bytes32::random(),
            input_index: 0,
            state_root: Bytes32::random(),
        });
        let slots = vec![StorageSlot {
            key: Bytes32::random(),
            value: Bytes32::random(),
        }];
        tx.storage_slots = Some(slots.clone());
        tx.storage_slots_count = slots.len() as u32;
        tx
    }

    fn with_mint_data(mut tx: Transaction) -> Transaction {
        tx.input_contract = Some(InputContract {
            balance_root: Bytes32::random(),
            contract_id: Bytes32::random(),
            state_root: Bytes32::random(),
            tx_pointer: TxPointer::random(),
            utxo_id: UtxoId::random(),
        });
        tx.mint_amount = Some(1000.into());
        tx.mint_asset_id = Some(AssetId::random());
        tx.mint_gas_price = Some(100.into());
        tx.tx_pointer = Some(TxPointer::random());
        tx.witnesses_count = 0; // Mint transactions typically have no witnesses
        tx
    }

    pub fn script(
        inputs: Vec<Input>,
        outputs: Vec<Output>,
        receipts: Vec<Receipt>,
    ) -> Transaction {
        let script = vec![1, 2, 3];
        let script_data = vec![4, 5, 6];
        let mut tx = Self::with_script_data(
            Self::base_transaction(TransactionType::Script),
            script.clone(),
            script_data.clone(),
        );
        tx.inputs = inputs.clone();
        tx.outputs = outputs.clone();
        tx.receipts = receipts;
        tx.inputs_count = inputs.len() as u32;
        tx.outputs_count = outputs.len() as u32;
        tx.witnesses_count =
            tx.witnesses.to_owned().map(|w| w.len() as u32).unwrap_or(0);
        tx
    }

    pub fn create(
        inputs: Vec<Input>,
        outputs: Vec<Output>,
        receipts: Vec<Receipt>,
    ) -> Transaction {
        let mut tx = Self::base_transaction(TransactionType::Create);
        tx.salt = Some(Salt::random());
        tx.inputs = inputs.clone();
        tx.outputs = outputs.clone();
        tx.receipts = receipts;
        tx.inputs_count = inputs.len() as u32;
        tx.outputs_count = outputs.len() as u32;
        tx.witnesses_count =
            tx.witnesses.to_owned().map(|w| w.len() as u32).unwrap_or(0);
        Self::with_contract_data(tx)
    }

    pub fn mint(
        inputs: Vec<Input>,
        outputs: Vec<Output>,
        receipts: Vec<Receipt>,
    ) -> Transaction {
        let mut tx = Self::base_transaction(TransactionType::Mint);
        tx.inputs = inputs.clone();
        tx.outputs = outputs.clone();
        tx.receipts = receipts;
        tx.inputs_count = inputs.len() as u32;
        tx.outputs_count = outputs.len() as u32;
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
                root: FuelCoreBytes32::zeroed(),
            }
            .into(),
        );
        tx.inputs = inputs.clone();
        tx.outputs = outputs.clone();
        tx.receipts = receipts;
        tx.inputs_count = inputs.len() as u32;
        tx.outputs_count = outputs.len() as u32;
        tx.witnesses_count =
            tx.witnesses.to_owned().map(|w| w.len() as u32).unwrap_or(0);
        tx
    }

    pub fn upload(
        inputs: Vec<Input>,
        outputs: Vec<Output>,
        receipts: Vec<Receipt>,
    ) -> Transaction {
        let mut tx = Self::base_transaction(TransactionType::Upload);
        tx.bytecode_root = Some(Bytes32::random());
        tx.bytecode_witness_index = Some(0);
        let proof_set = vec![Bytes32::random()];
        tx.proof_set = Some(proof_set.clone());
        tx.proof_set_count = proof_set.len() as u32;
        tx.subsection_index = Some(0);
        tx.subsections_number = Some(1);
        tx.inputs = inputs.clone();
        tx.outputs = outputs.clone();
        tx.receipts = receipts;
        tx.inputs_count = inputs.len() as u32;
        tx.outputs_count = outputs.len() as u32;
        tx.witnesses_count =
            tx.witnesses.to_owned().map(|w| w.len() as u32).unwrap_or(0);
        tx
    }

    pub fn blob(
        inputs: Vec<Input>,
        outputs: Vec<Output>,
        receipts: Vec<Receipt>,
    ) -> Transaction {
        let mut tx = Self::base_transaction(TransactionType::Blob);
        tx.blob_id = Some(BlobId::random());
        tx.inputs = inputs.clone();
        tx.outputs = outputs.clone();
        tx.receipts = receipts;
        tx.inputs_count = inputs.len() as u32;
        tx.outputs_count = outputs.len() as u32;
        tx.witnesses_count =
            tx.witnesses.to_owned().map(|w| w.len() as u32).unwrap_or(0);
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
        let original_tx = MockTransaction::script(
            MockInput::all(),
            MockOutput::all(),
            MockReceipt::all(),
        );

        let serialized = serde_json::to_string(&original_tx).unwrap();
        let deserialized: Transaction =
            serde_json::from_str(&serialized).unwrap();

        assert_eq!(deserialized, original_tx);

        let json_value: serde_json::Value =
            serde_json::from_str(&serialized).unwrap();
        assert_eq!(json_value["type"], json!("script"));
        assert_eq!(json_value["status"], json!("success"));

        for tx in MockTransaction::all() {
            let serialized = serde_json::to_string(&tx).unwrap();
            let deserialized: Transaction =
                serde_json::from_str(&serialized).unwrap();
            assert_eq!(deserialized, tx);

            let json_value: serde_json::Value =
                serde_json::from_str(&serialized).unwrap();
            assert_eq!(json_value["type"], json!(tx.r#type.to_string()));
        }
    }
}
