use crate::types::*;

// Output enum
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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
pub struct OutputCoin {
    pub amount: u64,
    pub asset_id: AssetId,
    pub to: Address,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct OutputChange {
    pub amount: u64,
    pub asset_id: AssetId,
    pub to: Address,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct OutputVariable {
    pub amount: u64,
    pub asset_id: AssetId,
    pub to: Address,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
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
pub struct OutputContractCreated {
    pub contract_id: ContractId,
    pub state_root: Bytes32,
}
