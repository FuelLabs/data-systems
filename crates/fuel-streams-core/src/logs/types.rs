use crate::types::*;

/// A convenient aggregate type to represent a Fuel logs to allow users
/// think about them agnostic of receipts.
#[derive(Debug, Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
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

impl From<Receipt> for Log {
    fn from(value: Receipt) -> Self {
        match value {
            Receipt::Log(log) => Log::WithoutData {
                id: log.id,
                ra: log.ra,
                rb: log.rb,
                rc: log.rc,
                rd: log.rd,
                pc: log.pc,
                is: log.is,
            },
            Receipt::LogData(log) => Log::WithData {
                id: log.id,
                ra: log.ra,
                rb: log.rb,
                ptr: log.ptr,
                len: log.len,
                digest: log.digest,
                pc: log.pc,
                is: log.is,
                data: log.data,
            },
            _ => panic!("Invalid receipt type"),
        }
    }
}
