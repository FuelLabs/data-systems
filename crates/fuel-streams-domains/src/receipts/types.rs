use fuel_streams_types::{fuel_core::*, primitives::*};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type")]
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

impl Receipt {
    #[cfg(any(test, feature = "test-helpers"))]
    pub fn as_call(&self) -> CallReceipt {
        match self {
            Receipt::Call(receipt) => receipt.clone(),
            _ => panic!("Invalid receipt type"),
        }
    }
}

// Individual Receipt Types
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CallReceipt {
    pub id: ContractId,
    pub to: ContractId,
    pub amount: FuelCoreWord,
    pub asset_id: AssetId,
    pub gas: FuelCoreWord,
    pub param1: FuelCoreWord,
    pub param2: FuelCoreWord,
    pub pc: FuelCoreWord,
    pub is: FuelCoreWord,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReturnReceipt {
    pub id: ContractId,
    pub val: FuelCoreWord,
    pub pc: FuelCoreWord,
    pub is: FuelCoreWord,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReturnDataReceipt {
    pub id: ContractId,
    pub ptr: FuelCoreWord,
    pub len: FuelCoreWord,
    pub digest: Bytes32,
    pub pc: FuelCoreWord,
    pub is: FuelCoreWord,
    pub data: Option<Vec<u8>>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PanicReceipt {
    pub id: ContractId,
    pub reason: PanicInstruction,
    pub pc: FuelCoreWord,
    pub is: FuelCoreWord,
    pub contract_id: Option<ContractId>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RevertReceipt {
    pub id: ContractId,
    pub ra: FuelCoreWord,
    pub pc: FuelCoreWord,
    pub is: FuelCoreWord,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LogReceipt {
    pub id: ContractId,
    pub ra: FuelCoreWord,
    pub rb: FuelCoreWord,
    pub rc: FuelCoreWord,
    pub rd: FuelCoreWord,
    pub pc: FuelCoreWord,
    pub is: FuelCoreWord,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LogDataReceipt {
    pub id: ContractId,
    pub ra: FuelCoreWord,
    pub rb: FuelCoreWord,
    pub ptr: FuelCoreWord,
    pub len: FuelCoreWord,
    pub digest: Bytes32,
    pub pc: FuelCoreWord,
    pub is: FuelCoreWord,
    pub data: Option<Vec<u8>>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TransferReceipt {
    pub id: ContractId,
    pub to: ContractId,
    pub amount: FuelCoreWord,
    pub asset_id: AssetId,
    pub pc: FuelCoreWord,
    pub is: FuelCoreWord,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TransferOutReceipt {
    pub id: ContractId,
    pub to: Address,
    pub amount: FuelCoreWord,
    pub asset_id: AssetId,
    pub pc: FuelCoreWord,
    pub is: FuelCoreWord,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScriptResultReceipt {
    pub result: ScriptExecutionResult,
    pub gas_used: FuelCoreWord,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MessageOutReceipt {
    pub sender: Address,
    pub recipient: Address,
    pub amount: FuelCoreWord,
    pub nonce: Nonce,
    pub len: FuelCoreWord,
    pub digest: Bytes32,
    pub data: Option<Vec<u8>>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MintReceipt {
    pub sub_id: Bytes32,
    pub contract_id: ContractId,
    pub val: FuelCoreWord,
    pub pc: FuelCoreWord,
    pub is: FuelCoreWord,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BurnReceipt {
    pub sub_id: Bytes32,
    pub contract_id: ContractId,
    pub val: FuelCoreWord,
    pub pc: FuelCoreWord,
    pub is: FuelCoreWord,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MockReceipt;
impl MockReceipt {
    pub fn call() -> Receipt {
        Receipt::Call(CallReceipt {
            id: ContractId::default(),
            to: ContractId::default(),
            amount: 100,
            asset_id: AssetId::default(),
            gas: 1000,
            param1: 0,
            param2: 0,
            pc: 0,
            is: 0,
        })
    }

    pub fn return_receipt() -> Receipt {
        Receipt::Return(ReturnReceipt {
            id: ContractId::default(),
            val: 0,
            pc: 0,
            is: 0,
        })
    }

    pub fn return_data() -> Receipt {
        Receipt::ReturnData(ReturnDataReceipt {
            id: ContractId::default(),
            ptr: 0,
            len: 0,
            digest: Bytes32::default(),
            pc: 0,
            is: 0,
            data: Some(vec![1, 2, 3]),
        })
    }

    pub fn panic() -> Receipt {
        Receipt::Panic(PanicReceipt {
            id: ContractId::default(),
            reason: PanicInstruction::default(),
            pc: 0,
            is: 0,
            contract_id: None,
        })
    }

    pub fn revert() -> Receipt {
        Receipt::Revert(RevertReceipt {
            id: ContractId::default(),
            ra: 0,
            pc: 0,
            is: 0,
        })
    }

    pub fn log() -> Receipt {
        Receipt::Log(LogReceipt {
            id: ContractId::default(),
            ra: 0,
            rb: 0,
            rc: 0,
            rd: 0,
            pc: 0,
            is: 0,
        })
    }

    pub fn log_data() -> Receipt {
        Receipt::LogData(LogDataReceipt {
            id: ContractId::default(),
            ra: 0,
            rb: 0,
            ptr: 0,
            len: 0,
            digest: Bytes32::default(),
            pc: 0,
            is: 0,
            data: Some(vec![4, 5, 6]),
        })
    }

    pub fn transfer() -> Receipt {
        Receipt::Transfer(TransferReceipt {
            id: ContractId::default(),
            to: ContractId::default(),
            amount: 100,
            asset_id: AssetId::default(),
            pc: 0,
            is: 0,
        })
    }

    pub fn transfer_out() -> Receipt {
        Receipt::TransferOut(TransferOutReceipt {
            id: ContractId::default(),
            to: Address::default(),
            amount: 100,
            asset_id: AssetId::default(),
            pc: 0,
            is: 0,
        })
    }

    pub fn script_result() -> Receipt {
        Receipt::ScriptResult(ScriptResultReceipt {
            result: ScriptExecutionResult::Success,
            gas_used: 1000,
        })
    }

    pub fn message_out() -> Receipt {
        Receipt::MessageOut(MessageOutReceipt {
            sender: Address::default(),
            recipient: Address::default(),
            amount: 100,
            nonce: Nonce::default(),
            len: 0,
            digest: Bytes32::default(),
            data: Some(vec![7, 8, 9]),
        })
    }

    pub fn mint() -> Receipt {
        Receipt::Mint(MintReceipt {
            sub_id: Bytes32::default(),
            contract_id: ContractId::default(),
            val: 100,
            pc: 0,
            is: 0,
        })
    }

    pub fn burn() -> Receipt {
        Receipt::Burn(BurnReceipt {
            sub_id: Bytes32::default(),
            contract_id: ContractId::default(),
            val: 100,
            pc: 0,
            is: 0,
        })
    }

    pub fn all() -> Vec<Receipt> {
        vec![
            Self::call(),
            Self::return_receipt(),
            Self::return_data(),
            Self::panic(),
            Self::revert(),
            Self::log(),
            Self::log_data(),
            Self::transfer(),
            Self::transfer_out(),
            Self::script_result(),
            Self::message_out(),
            Self::mint(),
            Self::burn(),
        ]
    }
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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ReceiptType {
    Call,
    Return,
    ReturnData,
    Panic,
    Revert,
    Log,
    LogData,
    Transfer,
    TransferOut,
    ScriptResult,
    MessageOut,
    Mint,
    Burn,
}

impl std::fmt::Display for ReceiptType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

impl From<ReceiptType> for String {
    fn from(value: ReceiptType) -> Self {
        match value {
            ReceiptType::Call => "call".to_string(),
            ReceiptType::Return => "return".to_string(),
            ReceiptType::ReturnData => "return_data".to_string(),
            ReceiptType::Panic => "panic".to_string(),
            ReceiptType::Revert => "revert".to_string(),
            ReceiptType::Log => "log".to_string(),
            ReceiptType::LogData => "log_data".to_string(),
            ReceiptType::Transfer => "transfer".to_string(),
            ReceiptType::TransferOut => "transfer_out".to_string(),
            ReceiptType::ScriptResult => "script_result".to_string(),
            ReceiptType::MessageOut => "message_out".to_string(),
            ReceiptType::Mint => "mint".to_string(),
            ReceiptType::Burn => "burn".to_string(),
        }
    }
}

impl std::str::FromStr for ReceiptType {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "call" => Ok(ReceiptType::Call),
            "return" => Ok(ReceiptType::Return),
            "return_data" => Ok(ReceiptType::ReturnData),
            "panic" => Ok(ReceiptType::Panic),
            "revert" => Ok(ReceiptType::Revert),
            "log" => Ok(ReceiptType::Log),
            "log_data" => Ok(ReceiptType::LogData),
            "transfer" => Ok(ReceiptType::Transfer),
            "transfer_out" => Ok(ReceiptType::TransferOut),
            "script_result" => Ok(ReceiptType::ScriptResult),
            "message_out" => Ok(ReceiptType::MessageOut),
            "mint" => Ok(ReceiptType::Mint),
            "burn" => Ok(ReceiptType::Burn),
            _ => Err(format!("Invalid receipt type: {}", s)),
        }
    }
}
