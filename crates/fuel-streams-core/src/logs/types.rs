use crate::types::*;

/// A convenient aggregate type to represent a Fuel logs to allow users
/// think about them agnostic of receipts.
#[derive(Debug, Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Log {
    WithoutData {
        id: ContractId,
        ra: FuelCoreWord,
        rb: FuelCoreWord,
        rc: FuelCoreWord,
        rd: FuelCoreWord,
        pc: FuelCoreWord,
        is: FuelCoreWord,
    },
    WithData {
        id: ContractId,
        ra: FuelCoreWord,
        rb: FuelCoreWord,
        ptr: FuelCoreWord,
        len: FuelCoreWord,
        digest: Bytes32,
        pc: FuelCoreWord,
        is: FuelCoreWord,
        data: Option<Vec<u8>>,
    },
}

impl From<FuelCoreReceipt> for Log {
    fn from(value: FuelCoreReceipt) -> Self {
        match value {
            FuelCoreReceipt::Log {
                id,
                ra,
                rb,
                rc,
                rd,
                pc,
                is,
            } => Log::WithoutData {
                id: id.into(),
                ra,
                rb,
                rc,
                rd,
                pc,
                is,
            },
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
            } => Log::WithData {
                id: id.into(),
                ra,
                rb,
                ptr,
                len,
                digest: digest.into(),
                pc,
                is,
                data,
            },
            _ => panic!("Invalid receipt type"),
        }
    }
}

/// Introduced majorly allow delegating serialization and deserialization to `fuel-core`'s Receipt
impl From<Log> for FuelCoreReceipt {
    fn from(log: Log) -> FuelCoreReceipt {
        match log {
            Log::WithoutData {
                id,
                ra,
                rb,
                rc,
                rd,
                pc,
                is,
            } => FuelCoreReceipt::Log {
                id: id.into(),
                ra,
                rb,
                rc,
                rd,
                pc,
                is,
            },
            Log::WithData {
                id,
                ra,
                rb,
                ptr,
                len,
                digest,
                pc,
                is,
                data,
            } => FuelCoreReceipt::LogData {
                id: id.into(),
                ra,
                rb,
                ptr,
                len,
                digest: digest.into(),
                pc,
                is,
                data,
            },
        }
    }
}
