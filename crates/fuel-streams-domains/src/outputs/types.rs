use fuel_streams_types::{fuel_core::*, primitives::*};
use serde::{Deserialize, Serialize};

// Output enum
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Output {
    Coin(OutputCoin),
    Contract(OutputContract),
    Change(OutputChange),
    Variable(OutputVariable),
    ContractCreated(OutputContractCreated),
}

impl From<&FuelCoreOutput> for Output {
    fn from(output: &FuelCoreOutput) -> Self {
        match output {
            FuelCoreOutput::Coin {
                amount,
                asset_id,
                to,
            } => Output::Coin(OutputCoin {
                amount: *amount,
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
                amount: *amount,
                asset_id: asset_id.into(),
                to: to.into(),
            }),
            FuelCoreOutput::Variable {
                amount,
                asset_id,
                to,
            } => Output::Variable(OutputVariable {
                amount: *amount,
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

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OutputCoin {
    pub amount: u64,
    pub asset_id: AssetId,
    pub to: Address,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OutputChange {
    pub amount: u64,
    pub asset_id: AssetId,
    pub to: Address,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OutputVariable {
    pub amount: u64,
    pub asset_id: AssetId,
    pub to: Address,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OutputContractCreated {
    pub contract_id: ContractId,
    pub state_root: Bytes32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MockOutput;
impl MockOutput {
    pub fn coin(amount: u64) -> Output {
        Output::Coin(OutputCoin {
            amount,
            asset_id: AssetId::default(),
            to: Address::default(),
        })
    }

    pub fn contract() -> Output {
        Output::Contract(OutputContract {
            balance_root: Bytes32::default(),
            input_index: 0,
            state_root: Bytes32::default(),
        })
    }

    pub fn change(amount: u64) -> Output {
        Output::Change(OutputChange {
            amount,
            asset_id: AssetId::default(),
            to: Address::default(),
        })
    }

    pub fn variable(amount: u64) -> Output {
        Output::Variable(OutputVariable {
            amount,
            asset_id: AssetId::default(),
            to: Address::default(),
        })
    }

    pub fn contract_created() -> Output {
        Output::ContractCreated(OutputContractCreated {
            contract_id: ContractId::default(),
            state_root: Bytes32::default(),
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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum OutputType {
    Coin,
    Contract,
    Change,
    Variable,
    ContractCreated,
}

impl std::fmt::Display for OutputType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OutputType::Coin => write!(f, "coin"),
            OutputType::Contract => write!(f, "contract"),
            OutputType::Change => write!(f, "change"),
            OutputType::Variable => write!(f, "variable"),
            OutputType::ContractCreated => write!(f, "contract_created"),
        }
    }
}

impl std::str::FromStr for OutputType {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "coin" => Ok(OutputType::Coin),
            "contract" => Ok(OutputType::Contract),
            "change" => Ok(OutputType::Change),
            "variable" => Ok(OutputType::Variable),
            "contract_created" => Ok(OutputType::ContractCreated),
            _ => Err(format!("Invalid output type: {}", s)),
        }
    }
}
