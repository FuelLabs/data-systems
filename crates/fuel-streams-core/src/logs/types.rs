use fuel_core_types::fuel_tx::{Bytes32, ContractId, Receipt, Word};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// A convenient aggregate type to represent a Fuel logs to allow users
/// think about them agnostic of receipts.
#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub enum Log {
    WithoutData {
        id: ContractId,
        ra: Word,
        rb: Word,
        rc: Word,
        rd: Word,
        pc: Word,
        is: Word,
    },
    WithData {
        id: ContractId,
        ra: Word,
        rb: Word,
        ptr: Word,
        len: Word,
        digest: Bytes32,
        pc: Word,
        is: Word,
        data: Option<Vec<u8>>,
    },
}

impl From<Receipt> for Log {
    fn from(value: Receipt) -> Self {
        match value {
            Receipt::Log {
                id,
                ra,
                rb,
                rc,
                rd,
                pc,
                is,
            } => Log::WithoutData {
                id,
                ra,
                rb,
                rc,
                rd,
                pc,
                is,
            },
            Receipt::LogData {
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
                id,
                ra,
                rb,
                ptr,
                len,
                digest,
                pc,
                is,
                data,
            },
            _ => panic!("Invalid receipt type"),
        }
    }
}

/// Introduced majorly allow delegating serialization and deserialization to `fuel-core`'s Receipt
impl From<Log> for Receipt {
    fn from(log: Log) -> Receipt {
        match log {
            Log::WithoutData {
                id,
                ra,
                rb,
                rc,
                rd,
                pc,
                is,
            } => Receipt::Log {
                id,
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
            } => Receipt::LogData {
                id,
                ra,
                rb,
                ptr,
                len,
                digest,
                pc,
                is,
                data,
            },
        }
    }
}

impl Serialize for Log {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let receipt: Receipt = self.clone().into();
        receipt.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for Log {
    fn deserialize<D>(deserializer: D) -> Result<Log, D::Error>
    where
        D: Deserializer<'de>,
    {
        let receipt = Receipt::deserialize(deserializer)?;
        Ok(Log::from(receipt))
    }
}
