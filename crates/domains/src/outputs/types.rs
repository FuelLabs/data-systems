use fuel_data_parser::DataEncoder;
use fuel_streams_types::{fuel_core::*, primitives::*};
use serde::{Deserialize, Serialize};

use crate::infra::record::ToPacket;

// Output enum
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, utoipa::ToSchema)]
#[serde(tag = "type")]
pub enum Output {
    Coin(OutputCoin),
    Contract(OutputContract),
    Change(OutputChange),
    Variable(OutputVariable),
    ContractCreated(OutputContractCreated),
}

impl DataEncoder for Output {}
impl ToPacket for Output {}

impl From<&FuelCoreOutput> for Output {
    fn from(output: &FuelCoreOutput) -> Self {
        match output {
            FuelCoreOutput::Coin {
                amount,
                asset_id,
                to,
            } => Output::Coin(OutputCoin {
                amount: (*amount).into(),
                asset_id: asset_id.into(),
                to: to.into(),
            }),
            FuelCoreOutput::Contract(contract) => {
                Output::Contract(contract.into())
            }
            FuelCoreOutput::Change {
                amount,
                asset_id,
                to,
            } => Output::Change(OutputChange {
                amount: (*amount).into(),
                asset_id: asset_id.into(),
                to: to.into(),
            }),
            FuelCoreOutput::Variable {
                amount,
                asset_id,
                to,
            } => Output::Variable(OutputVariable {
                amount: (*amount).into(),
                asset_id: asset_id.into(),
                to: to.into(),
            }),
            FuelCoreOutput::ContractCreated {
                contract_id,
                state_root,
            } => Output::ContractCreated(OutputContractCreated {
                contract_id: contract_id.into(),
                state_root: state_root.into(),
            }),
        }
    }
}

#[derive(
    Debug, Clone, Default, PartialEq, Serialize, Deserialize, utoipa::ToSchema,
)]
#[serde(rename_all = "camelCase")]
pub struct OutputCoin {
    pub amount: Amount,
    pub asset_id: AssetId,
    pub to: Address,
}

#[derive(
    Debug, Clone, Default, PartialEq, Serialize, Deserialize, utoipa::ToSchema,
)]
#[serde(rename_all = "camelCase")]
pub struct OutputChange {
    pub amount: Amount,
    pub asset_id: AssetId,
    pub to: Address,
}

#[derive(
    Debug, Clone, Default, PartialEq, Serialize, Deserialize, utoipa::ToSchema,
)]
#[serde(rename_all = "camelCase")]
pub struct OutputVariable {
    pub amount: Amount,
    pub asset_id: AssetId,
    pub to: Address,
}

#[derive(
    Debug, Clone, Default, PartialEq, Serialize, Deserialize, utoipa::ToSchema,
)]
#[serde(rename_all = "camelCase")]
pub struct OutputContract {
    pub balance_root: Bytes32,
    pub input_index: u16,
    pub state_root: Bytes32,
}

impl From<&FuelCoreOutputContract> for OutputContract {
    fn from(output: &FuelCoreOutputContract) -> Self {
        OutputContract {
            balance_root: output.balance_root.into(),
            input_index: output.input_index,
            state_root: output.state_root.into(),
        }
    }
}

#[derive(
    Debug, Clone, Default, PartialEq, Serialize, Deserialize, utoipa::ToSchema,
)]
#[serde(rename_all = "camelCase")]
pub struct OutputContractCreated {
    pub contract_id: ContractId,
    pub state_root: Bytes32,
}

#[cfg(any(test, feature = "test-helpers"))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MockOutput;
#[cfg(any(test, feature = "test-helpers"))]
impl MockOutput {
    pub fn coin(amount: u64) -> Output {
        Output::Coin(OutputCoin {
            amount: amount.into(),
            asset_id: AssetId::random(),
            to: Address::random(),
        })
    }

    pub fn contract() -> Output {
        Output::Contract(OutputContract {
            balance_root: Bytes32::random(),
            input_index: 0,
            state_root: Bytes32::random(),
        })
    }

    pub fn change(amount: u64) -> Output {
        Output::Change(OutputChange {
            amount: amount.into(),
            asset_id: AssetId::random(),
            to: Address::random(),
        })
    }

    pub fn variable(amount: u64) -> Output {
        Output::Variable(OutputVariable {
            amount: amount.into(),
            asset_id: AssetId::random(),
            to: Address::random(),
        })
    }

    pub fn contract_created() -> Output {
        Output::ContractCreated(OutputContractCreated {
            contract_id: ContractId::random(),
            state_root: Bytes32::random(),
        })
    }

    pub fn all() -> Vec<Output> {
        vec![
            Self::coin(1000),
            Self::contract(),
            Self::change(2000),
            Self::variable(3000),
            Self::contract_created(),
        ]
    }
}
