use fuel_core_types::fuel_asm::Word;
use serde::{self, Deserialize, Serialize};

use crate::types::*;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Receipt {
    Call(CallReceipt),
    Return(ReturnReceipt),
    ReturnData(ReturnDataReceipt),
    Panic(PanicReceipt),
    Revert(RevertReceipt),
    Log(LogReceipt),
    LogData(LogDataReceipt),
    Transfer(TransferReceipt),
    TransferOut(TransferOutReceipt),
    ScriptResult(ScriptResultReceipt),
    MessageOut(MessageOutReceipt),
    Mint(MintReceipt),
    Burn(BurnReceipt),
}

// Individual Receipt Types
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct CallReceipt {
    pub id: ContractId,
    pub to: ContractId,
    pub amount: Word,
    pub asset_id: AssetId,
    pub gas: Word,
    pub param1: Word,
    pub param2: Word,
    pub pc: Word,
    pub is: Word,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct ReturnReceipt {
    pub id: ContractId,
    pub val: Word,
    pub pc: Word,
    pub is: Word,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct ReturnDataReceipt {
    pub id: ContractId,
    pub ptr: Word,
    pub len: Word,
    pub digest: Bytes32,
    pub pc: Word,
    pub is: Word,
    pub data: Option<Vec<u8>>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct PanicReceipt {
    pub id: ContractId,
    pub reason: PanicInstruction,
    pub pc: Word,
    pub is: Word,
    pub contract_id: Option<ContractId>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct RevertReceipt {
    pub id: ContractId,
    pub ra: Word,
    pub pc: Word,
    pub is: Word,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct LogReceipt {
    pub id: ContractId,
    pub ra: Word,
    pub rb: Word,
    pub rc: Word,
    pub rd: Word,
    pub pc: Word,
    pub is: Word,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct LogDataReceipt {
    pub id: ContractId,
    pub ra: Word,
    pub rb: Word,
    pub ptr: Word,
    pub len: Word,
    pub digest: Bytes32,
    pub pc: Word,
    pub is: Word,
    pub data: Option<Vec<u8>>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct TransferReceipt {
    pub id: ContractId,
    pub to: ContractId,
    pub amount: Word,
    pub asset_id: AssetId,
    pub pc: Word,
    pub is: Word,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct TransferOutReceipt {
    pub id: ContractId,
    pub to: Address,
    pub amount: Word,
    pub asset_id: AssetId,
    pub pc: Word,
    pub is: Word,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct ScriptResultReceipt {
    pub result: ScriptExecutionResult,
    pub gas_used: Word,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct MessageOutReceipt {
    pub sender: Address,
    pub recipient: Address,
    pub amount: Word,
    pub nonce: Nonce,
    pub len: Word,
    pub digest: Bytes32,
    pub data: Option<Vec<u8>>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct MintReceipt {
    pub sub_id: Bytes32,
    pub contract_id: ContractId,
    pub val: Word,
    pub pc: Word,
    pub is: Word,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct BurnReceipt {
    pub sub_id: Bytes32,
    pub contract_id: ContractId,
    pub val: Word,
    pub pc: Word,
    pub is: Word,
}

impl From<FuelCoreReceipt> for Receipt {
    fn from(value: FuelCoreReceipt) -> Self {
        match value {
            FuelCoreReceipt::Call {
                id,
                to,
                amount,
                asset_id,
                gas,
                param1,
                param2,
                pc,
                is,
            } => Self::Call(CallReceipt {
                id: id.into(),
                to: to.into(),
                amount,
                asset_id: asset_id.into(),
                gas,
                param1,
                param2,
                pc,
                is,
            }),
            FuelCoreReceipt::Return { id, val, pc, is } => {
                Self::Return(ReturnReceipt {
                    id: id.into(),
                    val,
                    pc,
                    is,
                })
            }
            FuelCoreReceipt::ReturnData {
                id,
                ptr,
                len,
                digest,
                pc,
                is,
                data,
            } => Self::ReturnData(ReturnDataReceipt {
                id: id.into(),
                ptr,
                len,
                digest: digest.into(),
                pc,
                is,
                data,
            }),
            FuelCoreReceipt::Panic {
                id,
                reason,
                pc,
                is,
                contract_id,
            } => Self::Panic(PanicReceipt {
                id: id.into(),
                reason: reason.into(),
                pc,
                is,
                contract_id: contract_id.map(|id| id.into()),
            }),
            FuelCoreReceipt::Revert { id, ra, pc, is } => {
                Self::Revert(RevertReceipt {
                    id: id.into(),
                    ra,
                    pc,
                    is,
                })
            }
            FuelCoreReceipt::Log {
                id,
                ra,
                rb,
                rc,
                rd,
                pc,
                is,
            } => Self::Log(LogReceipt {
                id: id.into(),
                ra,
                rb,
                rc,
                rd,
                pc,
                is,
            }),
            FuelCoreReceipt::LogData {
                id,
                ra,
                rb,
                ptr,
                len,
                digest,
                pc,
                is,
                data,
            } => Self::LogData(LogDataReceipt {
                id: id.into(),
                ra,
                rb,
                ptr,
                len,
                digest: digest.into(),
                pc,
                is,
                data,
            }),
            FuelCoreReceipt::Transfer {
                id,
                to,
                amount,
                asset_id,
                pc,
                is,
            } => Self::Transfer(TransferReceipt {
                id: id.into(),
                to: to.into(),
                amount,
                asset_id: asset_id.into(),
                pc,
                is,
            }),
            FuelCoreReceipt::TransferOut {
                id,
                to,
                amount,
                asset_id,
                pc,
                is,
            } => Self::TransferOut(TransferOutReceipt {
                id: id.into(),
                to: to.into(),
                amount,
                asset_id: asset_id.into(),
                pc,
                is,
            }),
            FuelCoreReceipt::ScriptResult { result, gas_used } => {
                Self::ScriptResult(ScriptResultReceipt {
                    result: result.into(),
                    gas_used,
                })
            }
            FuelCoreReceipt::MessageOut {
                sender,
                recipient,
                amount,
                nonce,
                len,
                digest,
                data,
            } => Self::MessageOut(MessageOutReceipt {
                sender: sender.into(),
                recipient: recipient.into(),
                amount,
                nonce: nonce.into(),
                len,
                digest: digest.into(),
                data,
            }),
            FuelCoreReceipt::Mint {
                sub_id,
                contract_id,
                val,
                pc,
                is,
            } => Self::Mint(MintReceipt {
                sub_id: sub_id.into(),
                contract_id: contract_id.into(),
                val,
                pc,
                is,
            }),
            FuelCoreReceipt::Burn {
                sub_id,
                contract_id,
                val,
                pc,
                is,
            } => Self::Burn(BurnReceipt {
                sub_id: sub_id.into(),
                contract_id: contract_id.into(),
                val,
                pc,
                is,
            }),
        }
    }
}

impl From<&FuelCoreReceipt> for Receipt {
    fn from(value: &FuelCoreReceipt) -> Self {
        value.clone().into()
    }
}
