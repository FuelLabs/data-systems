use fuel_core_types::fuel_tx;
use fuel_streams_types::{fuel_core::*, primitives::*};
use serde::{Deserialize, Serialize};

use crate::{inputs::types::*, outputs::types::*, receipts::types::*};

#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
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

#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Transaction {
    pub id: TxId,
    #[serde(rename = "type")]
    pub kind: TransactionKind,
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
    pub mint_amount: Option<u64>,
    pub mint_asset_id: Option<AssetId>,
    pub mint_gas_price: Option<u64>,
    pub policies: Option<FuelCorePolicies>,
    pub proof_set: Vec<Bytes32>,
    pub raw_payload: HexData,
    pub receipts_root: Option<Bytes32>,
    pub salt: Option<Salt>,
    pub script: Option<HexData>,
    pub script_data: Option<HexData>,
    pub script_gas_limit: Option<u64>,
    pub status: TransactionStatus,
    pub storage_slots: Vec<StorageSlot>,
    pub subsection_index: Option<u16>,
    pub subsections_number: Option<u16>,
    pub tx_pointer: Option<FuelCoreTxPointer>,
    pub upgrade_purpose: Option<FuelCoreUpgradePurpose>,
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
            kind: transaction.into(),
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
            mint_amount,
            mint_asset_id,
            mint_gas_price,
            policies,
            proof_set,
            raw_payload,
            receipts_root,
            salt,
            script,
            script_data,
            script_gas_limit,
            status: status.to_owned(),
            storage_slots,
            subsection_index,
            subsections_number,
            tx_pointer,
            upgrade_purpose,
            witnesses,
            receipts: receipts.iter().map(|r| r.to_owned().into()).collect(),
        }
    }
}

#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum TransactionKind {
    #[default]
    Create,
    Mint,
    Script,
    Upgrade,
    Upload,
    Blob,
}

impl TransactionKind {
    fn as_str(&self) -> &'static str {
        match self {
            Self::Create => "create",
            Self::Mint => "mint",
            Self::Script => "script",
            Self::Upgrade => "upgrade",
            Self::Upload => "upload",
            Self::Blob => "blob",
        }
    }
}

impl std::fmt::Display for TransactionKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl From<&FuelCoreTransaction> for TransactionKind {
    fn from(value: &FuelCoreTransaction) -> Self {
        match value {
            FuelCoreTransaction::Script(_) => TransactionKind::Script,
            FuelCoreTransaction::Create(_) => TransactionKind::Create,
            FuelCoreTransaction::Mint(_) => TransactionKind::Mint,
            FuelCoreTransaction::Upgrade(_) => TransactionKind::Upgrade,
            FuelCoreTransaction::Upload(_) => TransactionKind::Upload,
            FuelCoreTransaction::Blob(_) => TransactionKind::Blob,
        }
    }
}

impl std::str::FromStr for TransactionKind {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            s if s == Self::Create.as_str() => Ok(Self::Create),
            s if s == Self::Mint.as_str() => Ok(Self::Mint),
            s if s == Self::Script.as_str() => Ok(Self::Script),
            s if s == Self::Upgrade.as_str() => Ok(Self::Upgrade),
            s if s == Self::Upload.as_str() => Ok(Self::Upload),
            s if s == Self::Blob.as_str() => Ok(Self::Blob),
            _ => Err(format!("Invalid transaction kind: {s}")),
        }
    }
}

#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "status")]
pub enum TransactionStatus {
    Failed,
    Submitted,
    SqueezedOut,
    Success,
    #[default]
    None,
}

impl std::fmt::Display for TransactionStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl TransactionStatus {
    fn as_str(&self) -> &'static str {
        match self {
            TransactionStatus::Failed => "failed",
            TransactionStatus::Submitted => "submitted",
            TransactionStatus::SqueezedOut => "squeezed_out",
            TransactionStatus::Success => "success",
            TransactionStatus::None => "none",
        }
    }
}

impl std::str::FromStr for TransactionStatus {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            s if s == Self::Failed.as_str() => Ok(Self::Failed),
            s if s == Self::Submitted.as_str() => Ok(Self::Submitted),
            s if s == Self::SqueezedOut.as_str() => Ok(Self::SqueezedOut),
            s if s == Self::Success.as_str() => Ok(Self::Success),
            s if s == Self::None.as_str() => Ok(Self::None),
            _ => Err(format!("Invalid transaction status: {s}")),
        }
    }
}

impl From<&FuelCoreTransactionStatus> for TransactionStatus {
    fn from(value: &FuelCoreTransactionStatus) -> Self {
        match value {
            FuelCoreTransactionStatus::Failed { .. } => {
                TransactionStatus::Failed
            }
            FuelCoreTransactionStatus::Submitted { .. } => {
                TransactionStatus::Submitted
            }
            FuelCoreTransactionStatus::SqueezedOut { .. } => {
                TransactionStatus::SqueezedOut
            }
            FuelCoreTransactionStatus::Success { .. } => {
                TransactionStatus::Success
            }
        }
    }
}

impl From<&FuelCoreClientTransactionStatus> for TransactionStatus {
    fn from(value: &FuelCoreClientTransactionStatus) -> Self {
        match value {
            FuelCoreClientTransactionStatus::Failure { .. } => {
                TransactionStatus::Failed
            }
            FuelCoreClientTransactionStatus::Submitted { .. } => {
                TransactionStatus::Submitted
            }
            FuelCoreClientTransactionStatus::SqueezedOut { .. } => {
                TransactionStatus::SqueezedOut
            }
            FuelCoreClientTransactionStatus::Success { .. } => {
                TransactionStatus::Success
            }
        }
    }
}

impl From<FuelCoreClientTransactionStatus> for TransactionStatus {
    fn from(value: FuelCoreClientTransactionStatus) -> Self {
        (&value).into()
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

#[derive(Debug, Clone)]
#[cfg(any(test, feature = "test-helpers"))]
pub struct MockTransaction(pub Transaction);

#[cfg(any(test, feature = "test-helpers"))]
impl MockTransaction {
    pub fn chain_id() -> FuelCoreChainId {
        FuelCoreChainId::default()
    }

    pub fn build() -> fuel_streams_types::FuelCoreTransaction {
        FuelCoreTransaction::default_test_tx()
    }
}
