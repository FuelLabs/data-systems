use std::cmp::Ordering;

use fuel_data_parser::DataEncoder;
use fuel_streams_types::{BlockHeight, BlockTimestamp, ReceiptType};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;

use super::{subjects::*, Receipt};
use crate::{
    infra::{
        db::DbItem,
        record::{
            RecordEntity,
            RecordPacket,
            RecordPacketError,
            RecordPointer,
        },
        Cursor,
        DbError,
    },
    Subjects,
};

#[derive(
    Debug, Clone, Serialize, Deserialize, PartialEq, Eq, sqlx::FromRow,
)]
pub struct ReceiptDbItem {
    pub subject: String,
    pub value: Vec<u8>,
    pub block_height: BlockHeight,
    pub tx_id: String,
    pub tx_index: i32,
    pub receipt_index: i32,

    // Common props
    pub r#type: ReceiptType,

    // call/transfer shared props
    pub from_contract_id: Option<String>, // 'id' in types
    pub to_contract_id: Option<String>,   // 'to' in types
    pub amount: Option<i64>,
    pub asset_id: Option<String>,
    pub gas: Option<i64>,    // call specific
    pub param1: Option<i64>, // call specific
    pub param2: Option<i64>, // call specific

    // return/return_data/panic/revert/log/log_data shared props
    pub contract_id: Option<String>, // 'id' in types
    pub pc: Option<i64>,
    pub is: Option<i64>,

    // return specific props
    pub val: Option<i64>,

    // return_data/log_data shared props
    pub ptr: Option<i64>,
    pub len: Option<i64>,
    pub digest: Option<String>,
    pub data: Option<String>,

    // log specific props
    pub ra: Option<i64>,
    pub rb: Option<i64>,
    pub rc: Option<i64>,
    pub rd: Option<i64>,

    // transfer_out specific props
    pub to_address: Option<String>, // 'to' in types for transfer_out

    // script_result specific props
    pub reason: Option<JsonValue>, /* panic specific: stores PanicInstruction {reason, instruction} */
    pub result: Option<String>,    // script_result specific
    pub gas_used: Option<i64>,

    // message_out specific props
    pub sender_address: Option<String>, // 'sender' in types
    pub recipient_address: Option<String>, // 'recipient' in types
    pub nonce: Option<String>,

    // mint/burn shared props
    pub sub_id: Option<String>,

    // timestamps
    pub block_time: BlockTimestamp,
    pub created_at: BlockTimestamp,
}

impl DataEncoder for ReceiptDbItem {}

impl DbItem for ReceiptDbItem {
    fn cursor(&self) -> Cursor {
        Cursor::new(&[&self.block_height, &self.tx_index, &self.receipt_index])
    }

    fn entity(&self) -> &RecordEntity {
        &RecordEntity::Receipt
    }

    fn encoded_value(&self) -> Result<Vec<u8>, DbError> {
        Ok(self.value.clone())
    }

    fn subject_str(&self) -> String {
        self.subject.clone()
    }

    fn subject_id(&self) -> String {
        match self.r#type {
            ReceiptType::Call => ReceiptsCallSubject::ID,
            ReceiptType::Return => ReceiptsReturnSubject::ID,
            ReceiptType::ReturnData => ReceiptsReturnDataSubject::ID,
            ReceiptType::Panic => ReceiptsPanicSubject::ID,
            ReceiptType::Revert => ReceiptsRevertSubject::ID,
            ReceiptType::Log => ReceiptsLogSubject::ID,
            ReceiptType::LogData => ReceiptsLogDataSubject::ID,
            ReceiptType::Transfer => ReceiptsTransferSubject::ID,
            ReceiptType::TransferOut => ReceiptsTransferOutSubject::ID,
            ReceiptType::ScriptResult => ReceiptsScriptResultSubject::ID,
            ReceiptType::MessageOut => ReceiptsMessageOutSubject::ID,
            ReceiptType::Mint => ReceiptsMintSubject::ID,
            ReceiptType::Burn => ReceiptsBurnSubject::ID,
        }
        .to_string()
    }

    fn created_at(&self) -> BlockTimestamp {
        self.created_at
    }

    fn block_time(&self) -> BlockTimestamp {
        self.block_time
    }

    fn block_height(&self) -> BlockHeight {
        self.block_height
    }
}

impl TryFrom<&RecordPacket> for ReceiptDbItem {
    type Error = RecordPacketError;
    fn try_from(packet: &RecordPacket) -> Result<Self, Self::Error> {
        let subject: Subjects = packet
            .subject_payload
            .to_owned()
            .try_into()
            .map_err(|_| RecordPacketError::SubjectMismatch)?;

        match subject {
            Subjects::ReceiptsCall(subject) => {
                let receipt = match Receipt::decode_json(&packet.value)? {
                    Receipt::Call(call) => call,
                    _ => return Err(RecordPacketError::SubjectMismatch),
                };

                Ok(ReceiptDbItem {
                    subject: packet.subject_str(),
                    value: packet.value.to_owned(),
                    block_height: subject.block_height.unwrap(),
                    tx_id: subject.tx_id.unwrap().to_string(),
                    tx_index: subject.tx_index.unwrap(),
                    receipt_index: subject.receipt_index.unwrap(),
                    r#type: ReceiptType::Call,
                    from_contract_id: Some(receipt.id.to_string()),
                    to_contract_id: Some(receipt.to.to_string()),
                    amount: Some(receipt.amount.into_inner() as i64),
                    asset_id: Some(receipt.asset_id.to_string()),
                    gas: Some(receipt.gas.into_inner() as i64),
                    param1: Some(receipt.param1.into_inner() as i64),
                    param2: Some(receipt.param2.into_inner() as i64),
                    pc: Some(receipt.pc.into_inner() as i64),
                    is: Some(receipt.is.into_inner() as i64),
                    contract_id: None,
                    val: None,
                    ptr: None,
                    len: None,
                    digest: None,
                    data: None,
                    ra: None,
                    rb: None,
                    rc: None,
                    rd: None,
                    to_address: None,
                    reason: None,
                    result: None,
                    gas_used: None,
                    sender_address: None,
                    recipient_address: None,
                    nonce: None,
                    sub_id: None,
                    block_time: packet.block_timestamp,
                    created_at: packet.block_timestamp,
                })
            }
            Subjects::ReceiptsReturn(subject) => {
                let receipt = match Receipt::decode_json(&packet.value)? {
                    Receipt::Return(ret) => ret,
                    _ => return Err(RecordPacketError::SubjectMismatch),
                };

                Ok(ReceiptDbItem {
                    subject: packet.subject_str(),
                    value: packet.value.to_owned(),
                    block_height: subject.block_height.unwrap(),
                    tx_id: subject.tx_id.unwrap().to_string(),
                    tx_index: subject.tx_index.unwrap(),
                    receipt_index: subject.receipt_index.unwrap(),
                    r#type: ReceiptType::Return,
                    contract_id: Some(receipt.id.to_string()),
                    val: Some(receipt.val.into_inner() as i64),
                    pc: Some(receipt.pc.into_inner() as i64),
                    is: Some(receipt.is.into_inner() as i64),
                    from_contract_id: None,
                    to_contract_id: None,
                    amount: None,
                    asset_id: None,
                    gas: None,
                    param1: None,
                    param2: None,
                    ptr: None,
                    len: None,
                    digest: None,
                    data: None,
                    ra: None,
                    rb: None,
                    rc: None,
                    rd: None,
                    to_address: None,
                    reason: None,
                    result: None,
                    gas_used: None,
                    sender_address: None,
                    recipient_address: None,
                    nonce: None,
                    sub_id: None,
                    block_time: packet.block_timestamp,
                    created_at: packet.block_timestamp,
                })
            }
            Subjects::ReceiptsReturnData(subject) => {
                let receipt = match Receipt::decode_json(&packet.value)? {
                    Receipt::ReturnData(ret_data) => ret_data,
                    _ => return Err(RecordPacketError::SubjectMismatch),
                };

                Ok(ReceiptDbItem {
                    subject: packet.subject_str(),
                    value: packet.value.to_owned(),
                    block_height: subject.block_height.unwrap(),
                    tx_id: subject.tx_id.unwrap().to_string(),
                    tx_index: subject.tx_index.unwrap(),
                    receipt_index: subject.receipt_index.unwrap(),
                    r#type: ReceiptType::ReturnData,
                    contract_id: Some(receipt.id.to_string()),
                    ptr: Some(receipt.ptr.into_inner() as i64),
                    len: Some(receipt.len.into_inner() as i64),
                    digest: Some(receipt.digest.to_string()),
                    data: receipt.data.map(|d| d.to_string()),
                    pc: Some(receipt.pc.into_inner() as i64),
                    is: Some(receipt.is.into_inner() as i64),
                    from_contract_id: None,
                    to_contract_id: None,
                    amount: None,
                    asset_id: None,
                    gas: None,
                    param1: None,
                    param2: None,
                    val: None,
                    ra: None,
                    rb: None,
                    rc: None,
                    rd: None,
                    to_address: None,
                    reason: None,
                    result: None,
                    gas_used: None,
                    sender_address: None,
                    recipient_address: None,
                    nonce: None,
                    sub_id: None,
                    block_time: packet.block_timestamp,
                    created_at: packet.block_timestamp,
                })
            }
            Subjects::ReceiptsPanic(subject) => {
                let receipt = match Receipt::decode_json(&packet.value)? {
                    Receipt::Panic(panic) => panic,
                    _ => return Err(RecordPacketError::SubjectMismatch),
                };

                Ok(ReceiptDbItem {
                    subject: packet.subject_str(),
                    value: packet.value.to_owned(),
                    block_height: subject.block_height.unwrap(),
                    tx_id: subject.tx_id.unwrap().to_string(),
                    tx_index: subject.tx_index.unwrap(),
                    receipt_index: subject.receipt_index.unwrap(),
                    r#type: ReceiptType::Panic,
                    contract_id: Some(receipt.id.to_string()),
                    reason: Some(serde_json::to_value(&receipt.reason)?),
                    pc: Some(receipt.pc.into_inner() as i64),
                    is: Some(receipt.is.into_inner() as i64),
                    from_contract_id: None,
                    to_contract_id: None,
                    amount: None,
                    asset_id: None,
                    gas: None,
                    param1: None,
                    param2: None,
                    val: None,
                    ptr: None,
                    len: None,
                    digest: None,
                    data: None,
                    ra: None,
                    rb: None,
                    rc: None,
                    rd: None,
                    to_address: None,
                    result: None,
                    gas_used: None,
                    sender_address: None,
                    recipient_address: None,
                    nonce: None,
                    sub_id: None,
                    block_time: packet.block_timestamp,
                    created_at: packet.block_timestamp,
                })
            }
            Subjects::ReceiptsRevert(subject) => {
                let receipt = match Receipt::decode_json(&packet.value)? {
                    Receipt::Revert(revert) => revert,
                    _ => return Err(RecordPacketError::SubjectMismatch),
                };

                Ok(ReceiptDbItem {
                    subject: packet.subject_str(),
                    value: packet.value.to_owned(),
                    block_height: subject.block_height.unwrap(),
                    tx_id: subject.tx_id.unwrap().to_string(),
                    tx_index: subject.tx_index.unwrap(),
                    receipt_index: subject.receipt_index.unwrap(),
                    r#type: ReceiptType::Revert,
                    contract_id: Some(receipt.id.to_string()),
                    ra: Some(receipt.ra.into_inner() as i64),
                    pc: Some(receipt.pc.into_inner() as i64),
                    is: Some(receipt.is.into_inner() as i64),
                    from_contract_id: None,
                    to_contract_id: None,
                    amount: None,
                    asset_id: None,
                    gas: None,
                    param1: None,
                    param2: None,
                    val: None,
                    ptr: None,
                    len: None,
                    digest: None,
                    data: None,
                    rb: None,
                    rc: None,
                    rd: None,
                    to_address: None,
                    reason: None,
                    result: None,
                    gas_used: None,
                    sender_address: None,
                    recipient_address: None,
                    nonce: None,
                    sub_id: None,
                    block_time: packet.block_timestamp,
                    created_at: packet.block_timestamp,
                })
            }
            Subjects::ReceiptsLog(subject) => {
                let receipt = match Receipt::decode_json(&packet.value)? {
                    Receipt::Log(log) => log,
                    _ => return Err(RecordPacketError::SubjectMismatch),
                };

                Ok(ReceiptDbItem {
                    subject: packet.subject_str(),
                    value: packet.value.to_owned(),
                    block_height: subject.block_height.unwrap(),
                    tx_id: subject.tx_id.unwrap().to_string(),
                    tx_index: subject.tx_index.unwrap(),
                    receipt_index: subject.receipt_index.unwrap(),
                    r#type: ReceiptType::Log,
                    contract_id: Some(receipt.id.to_string()),
                    ra: Some(receipt.ra.into_inner() as i64),
                    rb: Some(receipt.rb.into_inner() as i64),
                    rc: Some(receipt.rc.into_inner() as i64),
                    rd: Some(receipt.rd.into_inner() as i64),
                    pc: Some(receipt.pc.into_inner() as i64),
                    is: Some(receipt.is.into_inner() as i64),
                    from_contract_id: None,
                    to_contract_id: None,
                    amount: None,
                    asset_id: None,
                    gas: None,
                    param1: None,
                    param2: None,
                    val: None,
                    ptr: None,
                    len: None,
                    digest: None,
                    data: None,
                    to_address: None,
                    reason: None,
                    result: None,
                    gas_used: None,
                    sender_address: None,
                    recipient_address: None,
                    nonce: None,
                    sub_id: None,
                    block_time: packet.block_timestamp,
                    created_at: packet.block_timestamp,
                })
            }
            Subjects::ReceiptsLogData(subject) => {
                let receipt = match Receipt::decode_json(&packet.value)? {
                    Receipt::LogData(log_data) => log_data,
                    _ => return Err(RecordPacketError::SubjectMismatch),
                };

                Ok(ReceiptDbItem {
                    subject: packet.subject_str(),
                    value: packet.value.to_owned(),
                    block_height: subject.block_height.unwrap(),
                    tx_id: subject.tx_id.unwrap().to_string(),
                    tx_index: subject.tx_index.unwrap(),
                    receipt_index: subject.receipt_index.unwrap(),
                    r#type: ReceiptType::LogData,
                    contract_id: Some(receipt.id.to_string()),
                    ra: Some(receipt.ra.into_inner() as i64),
                    rb: Some(receipt.rb.into_inner() as i64),
                    ptr: Some(receipt.ptr.into_inner() as i64),
                    len: Some(receipt.len.into_inner() as i64),
                    digest: Some(receipt.digest.to_string()),
                    data: receipt.data.map(|d| d.to_string()),
                    pc: Some(receipt.pc.into_inner() as i64),
                    is: Some(receipt.is.into_inner() as i64),
                    from_contract_id: None,
                    to_contract_id: None,
                    amount: None,
                    asset_id: None,
                    gas: None,
                    param1: None,
                    param2: None,
                    val: None,
                    rc: None,
                    rd: None,
                    to_address: None,
                    reason: None,
                    result: None,
                    gas_used: None,
                    sender_address: None,
                    recipient_address: None,
                    nonce: None,
                    sub_id: None,
                    block_time: packet.block_timestamp,
                    created_at: packet.block_timestamp,
                })
            }
            Subjects::ReceiptsTransfer(subject) => {
                let receipt = match Receipt::decode_json(&packet.value)? {
                    Receipt::Transfer(transfer) => transfer,
                    _ => return Err(RecordPacketError::SubjectMismatch),
                };

                Ok(ReceiptDbItem {
                    subject: packet.subject_str(),
                    value: packet.value.to_owned(),
                    block_height: subject.block_height.unwrap(),
                    tx_id: subject.tx_id.unwrap().to_string(),
                    tx_index: subject.tx_index.unwrap(),
                    receipt_index: subject.receipt_index.unwrap(),
                    r#type: ReceiptType::Transfer,
                    from_contract_id: Some(receipt.id.to_string()),
                    to_contract_id: Some(receipt.to.to_string()),
                    amount: Some(receipt.amount.into_inner() as i64),
                    asset_id: Some(receipt.asset_id.to_string()),
                    pc: Some(receipt.pc.into_inner() as i64),
                    is: Some(receipt.is.into_inner() as i64),
                    contract_id: None,
                    gas: None,
                    param1: None,
                    param2: None,
                    val: None,
                    ptr: None,
                    len: None,
                    digest: None,
                    data: None,
                    ra: None,
                    rb: None,
                    rc: None,
                    rd: None,
                    to_address: None,
                    reason: None,
                    result: None,
                    gas_used: None,
                    sender_address: None,
                    recipient_address: None,
                    nonce: None,
                    sub_id: None,
                    block_time: packet.block_timestamp,
                    created_at: packet.block_timestamp,
                })
            }
            Subjects::ReceiptsTransferOut(subject) => {
                let receipt = match Receipt::decode_json(&packet.value)? {
                    Receipt::TransferOut(transfer_out) => transfer_out,
                    _ => return Err(RecordPacketError::SubjectMismatch),
                };

                Ok(ReceiptDbItem {
                    subject: packet.subject_str(),
                    value: packet.value.to_owned(),
                    block_height: subject.block_height.unwrap(),
                    tx_id: subject.tx_id.unwrap().to_string(),
                    tx_index: subject.tx_index.unwrap(),
                    receipt_index: subject.receipt_index.unwrap(),
                    r#type: ReceiptType::TransferOut,
                    from_contract_id: Some(receipt.id.to_string()),
                    to_address: Some(receipt.to.to_string()),
                    amount: Some(receipt.amount.into_inner() as i64),
                    asset_id: Some(receipt.asset_id.to_string()),
                    pc: Some(receipt.pc.into_inner() as i64),
                    is: Some(receipt.is.into_inner() as i64),
                    contract_id: None,
                    to_contract_id: None,
                    gas: None,
                    param1: None,
                    param2: None,
                    val: None,
                    ptr: None,
                    len: None,
                    digest: None,
                    data: None,
                    ra: None,
                    rb: None,
                    rc: None,
                    rd: None,
                    reason: None,
                    result: None,
                    gas_used: None,
                    sender_address: None,
                    recipient_address: None,
                    nonce: None,
                    sub_id: None,
                    block_time: packet.block_timestamp,
                    created_at: packet.block_timestamp,
                })
            }
            Subjects::ReceiptsScriptResult(subject) => {
                let receipt = match Receipt::decode_json(&packet.value)? {
                    Receipt::ScriptResult(script_result) => script_result,
                    _ => return Err(RecordPacketError::SubjectMismatch),
                };

                Ok(ReceiptDbItem {
                    subject: packet.subject_str(),
                    value: packet.value.to_owned(),
                    block_height: subject.block_height.unwrap(),
                    tx_id: subject.tx_id.unwrap().to_string(),
                    tx_index: subject.tx_index.unwrap(),
                    receipt_index: subject.receipt_index.unwrap(),
                    r#type: ReceiptType::ScriptResult,
                    reason: None,
                    result: Some(receipt.result.to_string()),
                    gas_used: Some(receipt.gas_used.into_inner() as i64),
                    from_contract_id: None,
                    to_contract_id: None,
                    amount: None,
                    asset_id: None,
                    gas: None,
                    param1: None,
                    param2: None,
                    contract_id: None,
                    pc: None,
                    is: None,
                    val: None,
                    ptr: None,
                    len: None,
                    digest: None,
                    data: None,
                    ra: None,
                    rb: None,
                    rc: None,
                    rd: None,
                    to_address: None,
                    sender_address: None,
                    recipient_address: None,
                    nonce: None,
                    sub_id: None,
                    block_time: packet.block_timestamp,
                    created_at: packet.block_timestamp,
                })
            }
            Subjects::ReceiptsMessageOut(subject) => {
                let receipt = match Receipt::decode_json(&packet.value)? {
                    Receipt::MessageOut(message_out) => message_out,
                    _ => return Err(RecordPacketError::SubjectMismatch),
                };

                Ok(ReceiptDbItem {
                    subject: packet.subject_str(),
                    value: packet.value.to_owned(),
                    block_height: subject.block_height.unwrap(),
                    tx_id: subject.tx_id.unwrap().to_string(),
                    tx_index: subject.tx_index.unwrap(),
                    receipt_index: subject.receipt_index.unwrap(),
                    r#type: ReceiptType::MessageOut,
                    sender_address: Some(receipt.sender.to_string()),
                    recipient_address: Some(receipt.recipient.to_string()),
                    amount: Some(receipt.amount.into_inner() as i64),
                    nonce: Some(receipt.nonce.to_string()),
                    len: Some(receipt.len.into_inner() as i64),
                    digest: Some(receipt.digest.to_string()),
                    data: receipt.data.map(|d| d.to_string()),
                    from_contract_id: None,
                    to_contract_id: None,
                    asset_id: None,
                    gas: None,
                    param1: None,
                    param2: None,
                    contract_id: None,
                    pc: None,
                    is: None,
                    val: None,
                    ptr: None,
                    ra: None,
                    rb: None,
                    rc: None,
                    rd: None,
                    to_address: None,
                    reason: None,
                    result: None,
                    gas_used: None,
                    sub_id: None,
                    block_time: packet.block_timestamp,
                    created_at: packet.block_timestamp,
                })
            }
            Subjects::ReceiptsMint(subject) => {
                let receipt = match Receipt::decode_json(&packet.value)? {
                    Receipt::Mint(mint) => mint,
                    _ => return Err(RecordPacketError::SubjectMismatch),
                };

                Ok(ReceiptDbItem {
                    subject: packet.subject_str(),
                    value: packet.value.to_owned(),
                    block_height: subject.block_height.unwrap(),
                    tx_id: subject.tx_id.unwrap().to_string(),
                    tx_index: subject.tx_index.unwrap(),
                    receipt_index: subject.receipt_index.unwrap(),
                    r#type: ReceiptType::Mint,
                    sub_id: Some(receipt.sub_id.to_string()),
                    contract_id: Some(receipt.contract_id.to_string()),
                    val: Some(receipt.val.into_inner() as i64),
                    pc: Some(receipt.pc.into_inner() as i64),
                    is: Some(receipt.is.into_inner() as i64),
                    from_contract_id: None,
                    to_contract_id: None,
                    amount: None,
                    asset_id: None,
                    gas: None,
                    param1: None,
                    param2: None,
                    ptr: None,
                    len: None,
                    digest: None,
                    data: None,
                    ra: None,
                    rb: None,
                    rc: None,
                    rd: None,
                    to_address: None,
                    reason: None,
                    result: None,
                    gas_used: None,
                    sender_address: None,
                    recipient_address: None,
                    nonce: None,
                    block_time: packet.block_timestamp,
                    created_at: packet.block_timestamp,
                })
            }
            Subjects::ReceiptsBurn(subject) => {
                let receipt = match Receipt::decode_json(&packet.value)? {
                    Receipt::Burn(burn) => burn,
                    _ => return Err(RecordPacketError::SubjectMismatch),
                };

                Ok(ReceiptDbItem {
                    subject: packet.subject_str(),
                    value: packet.value.to_owned(),
                    block_height: subject.block_height.unwrap(),
                    tx_id: subject.tx_id.unwrap().to_string(),
                    tx_index: subject.tx_index.unwrap(),
                    receipt_index: subject.receipt_index.unwrap(),
                    r#type: ReceiptType::Burn,
                    sub_id: Some(receipt.sub_id.to_string()),
                    contract_id: Some(receipt.contract_id.to_string()),
                    val: Some(receipt.val.into_inner() as i64),
                    pc: Some(receipt.pc.into_inner() as i64),
                    is: Some(receipt.is.into_inner() as i64),
                    from_contract_id: None,
                    to_contract_id: None,
                    amount: None,
                    asset_id: None,
                    gas: None,
                    param1: None,
                    param2: None,
                    ptr: None,
                    len: None,
                    digest: None,
                    data: None,
                    ra: None,
                    rb: None,
                    rc: None,
                    rd: None,
                    to_address: None,
                    reason: None,
                    result: None,
                    gas_used: None,
                    sender_address: None,
                    recipient_address: None,
                    nonce: None,
                    block_time: packet.block_timestamp,
                    created_at: packet.block_timestamp,
                })
            }
            _ => Err(RecordPacketError::SubjectMismatch),
        }
    }
}

impl PartialOrd for ReceiptDbItem {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for ReceiptDbItem {
    fn cmp(&self, other: &Self) -> Ordering {
        self.block_height
            .cmp(&other.block_height)
            .then(self.tx_index.cmp(&other.tx_index))
            .then(self.receipt_index.cmp(&other.receipt_index))
    }
}

impl From<ReceiptDbItem> for RecordPointer {
    fn from(val: ReceiptDbItem) -> Self {
        RecordPointer {
            block_height: val.block_height,
            tx_index: Some(val.tx_index as u32),
            input_index: None,
            output_index: None,
            receipt_index: Some(val.receipt_index as u32),
        }
    }
}
