use crate::types::*;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Receipt {
    pub amount: Option<u64>,
    pub asset_id: Option<AssetId>,
    pub contract_id: Option<ContractId>,
    pub data: Option<HexString>,
    pub digest: Option<Bytes32>,
    pub gas: Option<u64>,
    pub gas_used: Option<u64>,
    pub id: Option<ContractId>,
    pub is: Option<u64>,
    pub len: Option<u64>,
    pub nonce: Option<Nonce>,
    pub param1: Option<u64>,
    pub param2: Option<u64>,
    pub pc: Option<u64>,
    pub ptr: Option<u64>,
    pub ra: Option<u64>,
    pub rb: Option<u64>,
    pub rc: Option<u64>,
    pub rd: Option<u64>,
    pub reason: Option<u64>,
    pub receipt_type: ReceiptType,
    pub recipient: Option<Address>,
    pub result: Option<u64>,
    pub sender: Option<Address>,
    pub sub_id: Option<Bytes32>,
    pub to: Option<ContractId>,
    pub to_address: Option<Address>,
    pub val: Option<u64>,
}

impl From<&FuelCoreReceipt> for Receipt {
    fn from(r: &FuelCoreReceipt) -> Self {
        Receipt {
            amount: r.amount().map(Into::into),
            asset_id: r.asset_id().copied().map(Into::into),
            contract_id: r.contract_id().map(Into::into),
            data: r.data().map(Into::into),
            digest: r.digest().copied().map(Into::into),
            gas: r.gas().map(Into::into),
            gas_used: r.gas_used().map(Into::into),
            id: r.id().map(Into::into),
            is: r.is().map(Into::into),
            len: r.len().map(Into::into),
            nonce: r.nonce().copied().map(Into::into),
            param1: r.param1().map(Into::into),
            param2: r.param2().map(Into::into),
            pc: r.pc().map(Into::into),
            ptr: r.ptr().map(Into::into),
            ra: r.ra().map(Into::into),
            rb: r.rb().map(Into::into),
            rc: r.rc().map(Into::into),
            rd: r.rd().map(Into::into),
            reason: r.reason().map(Into::into),
            receipt_type: r.into(),
            recipient: r.recipient().copied().map(Into::into),
            result: r.result().map(|r| FuelCoreWord::from(*r)),
            sender: r.sender().copied().map(Into::into),
            sub_id: r.sub_id().copied().map(Into::into),
            to: r.to().copied().map(Into::into),
            to_address: r.to_address().copied().map(Into::into),
            val: r.val().map(Into::into),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ReceiptType {
    Burn,
    Call,
    Log,
    LogData,
    MessageOut,
    Mint,
    Panic,
    Return,
    ReturnData,
    Revert,
    ScriptResult,
    Transfer,
    TransferOut,
}

impl From<&FuelCoreReceipt> for ReceiptType {
    fn from(r: &FuelCoreReceipt) -> Self {
        match r {
            FuelCoreReceipt::Call { .. } => ReceiptType::Call,
            FuelCoreReceipt::Return { .. } => ReceiptType::Return,
            FuelCoreReceipt::ReturnData { .. } => ReceiptType::ReturnData,
            FuelCoreReceipt::Panic { .. } => ReceiptType::Panic,
            FuelCoreReceipt::Revert { .. } => ReceiptType::Revert,
            FuelCoreReceipt::Log { .. } => ReceiptType::Log,
            FuelCoreReceipt::LogData { .. } => ReceiptType::LogData,
            FuelCoreReceipt::Transfer { .. } => ReceiptType::Transfer,
            FuelCoreReceipt::TransferOut { .. } => ReceiptType::TransferOut,
            FuelCoreReceipt::ScriptResult { .. } => ReceiptType::ScriptResult,
            FuelCoreReceipt::MessageOut { .. } => ReceiptType::MessageOut,
            FuelCoreReceipt::Mint { .. } => ReceiptType::Mint,
            FuelCoreReceipt::Burn { .. } => ReceiptType::Burn,
        }
    }
}
