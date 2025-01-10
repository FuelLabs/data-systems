use fuel_streams_macros::subject::*;
use fuel_streams_types::*;
use serde::{Deserialize, Serialize};

use crate::blocks::types::*;

#[derive(Subject, Debug, Clone, Default, Serialize, Deserialize)]
#[subject_id = "receipts_call"]
#[subject_wildcard = "receipts.>"]
#[subject_format = "receipts.call.{block_height}.{tx_id}.{tx_index}.{receipt_index}.{from}.{to}.{asset_id}"]
pub struct ReceiptsCallSubject {
    pub block_height: Option<BlockHeight>,
    pub tx_id: Option<TxId>,
    pub tx_index: Option<u32>,
    pub receipt_index: Option<u32>,
    pub from: Option<ContractId>,
    pub to: Option<ContractId>,
    pub asset_id: Option<AssetId>,
}

#[derive(Subject, Debug, Clone, Default, Serialize, Deserialize)]
#[subject_id = "receipts_return"]
#[subject_wildcard = "receipts.>"]
#[subject_format = "receipts.return.{block_height}.{tx_id}.{tx_index}.{receipt_index}.{id}"]
pub struct ReceiptsReturnSubject {
    pub block_height: Option<BlockHeight>,
    pub tx_id: Option<TxId>,
    pub tx_index: Option<u32>,
    pub receipt_index: Option<u32>,
    pub id: Option<ContractId>,
}

#[derive(Subject, Debug, Clone, Default, Serialize, Deserialize)]
#[subject_id = "receipts_return_data"]
#[subject_wildcard = "receipts.>"]
#[subject_format = "receipts.return_data.{block_height}.{tx_id}.{tx_index}.{receipt_index}.{id}"]
pub struct ReceiptsReturnDataSubject {
    pub block_height: Option<BlockHeight>,
    pub tx_id: Option<TxId>,
    pub tx_index: Option<u32>,
    pub receipt_index: Option<u32>,
    pub id: Option<ContractId>,
}

#[derive(Subject, Debug, Clone, Default, Serialize, Deserialize)]
#[subject_id = "receipts_panic"]
#[subject_wildcard = "receipts.>"]
#[subject_format = "receipts.panic.{block_height}.{tx_id}.{tx_index}.{receipt_index}.{id}"]
pub struct ReceiptsPanicSubject {
    pub block_height: Option<BlockHeight>,
    pub tx_id: Option<TxId>,
    pub tx_index: Option<u32>,
    pub receipt_index: Option<u32>,
    pub id: Option<ContractId>,
}

#[derive(Subject, Debug, Clone, Default, Serialize, Deserialize)]
#[subject_id = "receipts_revert"]
#[subject_wildcard = "receipts.>"]
#[subject_format = "receipts.revert.{block_height}.{tx_id}.{tx_index}.{receipt_index}.{id}"]
pub struct ReceiptsRevertSubject {
    pub block_height: Option<BlockHeight>,
    pub tx_id: Option<TxId>,
    pub tx_index: Option<u32>,
    pub receipt_index: Option<u32>,
    pub id: Option<ContractId>,
}

#[derive(Subject, Debug, Clone, Default, Serialize, Deserialize)]
#[subject_id = "receipts_log"]
#[subject_wildcard = "receipts.>"]
#[subject_format = "receipts.log.{block_height}.{tx_id}.{tx_index}.{receipt_index}.{id}"]
pub struct ReceiptsLogSubject {
    pub block_height: Option<BlockHeight>,
    pub tx_id: Option<TxId>,
    pub tx_index: Option<u32>,
    pub receipt_index: Option<u32>,
    pub id: Option<ContractId>,
}

#[derive(Subject, Debug, Clone, Default, Serialize, Deserialize)]
#[subject_id = "receipts_log_data"]
#[subject_wildcard = "receipts.>"]
#[subject_format = "receipts.log_data.{block_height}.{tx_id}.{tx_index}.{receipt_index}.{id}"]
pub struct ReceiptsLogDataSubject {
    pub block_height: Option<BlockHeight>,
    pub tx_id: Option<TxId>,
    pub tx_index: Option<u32>,
    pub receipt_index: Option<u32>,
    pub id: Option<ContractId>,
}

#[derive(Subject, Debug, Clone, Default, Serialize, Deserialize)]
#[subject_id = "receipts_transfer"]
#[subject_wildcard = "receipts.>"]
#[subject_format = "receipts.transfer.{block_height}.{tx_id}.{tx_index}.{receipt_index}.{from}.{to}.{asset_id}"]
pub struct ReceiptsTransferSubject {
    pub block_height: Option<BlockHeight>,
    pub tx_id: Option<TxId>,
    pub tx_index: Option<u32>,
    pub receipt_index: Option<u32>,
    pub from: Option<ContractId>,
    pub to: Option<ContractId>,
    pub asset_id: Option<AssetId>,
}

#[derive(Subject, Debug, Clone, Default, Serialize, Deserialize)]
#[subject_id = "receipts_transfer_out"]
#[subject_wildcard = "receipts.>"]
#[subject_format = "receipts.transfer_out.{block_height}.{tx_id}.{tx_index}.{receipt_index}.{from}.{to}.{asset_id}"]
pub struct ReceiptsTransferOutSubject {
    pub block_height: Option<BlockHeight>,
    pub tx_id: Option<TxId>,
    pub tx_index: Option<u32>,
    pub receipt_index: Option<u32>,
    pub from: Option<ContractId>,
    pub to: Option<Address>,
    pub asset_id: Option<AssetId>,
}

#[derive(Subject, Debug, Clone, Default, Serialize, Deserialize)]
#[subject_id = "receipts_script_result"]
#[subject_wildcard = "receipts.>"]
#[subject_format = "receipts.script_result.{block_height}.{tx_id}.{tx_index}.{receipt_index}"]
pub struct ReceiptsScriptResultSubject {
    pub block_height: Option<BlockHeight>,
    pub tx_id: Option<TxId>,
    pub tx_index: Option<u32>,
    pub receipt_index: Option<u32>,
}

#[derive(Subject, Debug, Clone, Default, Serialize, Deserialize)]
#[subject_id = "receipts_message_out"]
#[subject_wildcard = "receipts.>"]
#[subject_format = "receipts.message_out.{block_height}.{tx_id}.{tx_index}.{receipt_index}.{sender}.{recipient}"]
pub struct ReceiptsMessageOutSubject {
    pub block_height: Option<BlockHeight>,
    pub tx_id: Option<TxId>,
    pub tx_index: Option<u32>,
    pub receipt_index: Option<u32>,
    pub sender: Option<Address>,
    pub recipient: Option<Address>,
}

#[derive(Subject, Debug, Clone, Default, Serialize, Deserialize)]
#[subject_id = "receipts_mint"]
#[subject_wildcard = "receipts.>"]
#[subject_format = "receipts.mint.{block_height}.{tx_id}.{tx_index}.{receipt_index}.{contract_id}.{sub_id}"]
pub struct ReceiptsMintSubject {
    pub block_height: Option<BlockHeight>,
    pub tx_id: Option<TxId>,
    pub tx_index: Option<u32>,
    pub receipt_index: Option<u32>,
    pub contract_id: Option<ContractId>,
    pub sub_id: Option<Bytes32>,
}

#[derive(Subject, Debug, Clone, Default, Serialize, Deserialize)]
#[subject_id = "receipts_burn"]
#[subject_wildcard = "receipts.>"]
#[subject_format = "receipts.burn.{block_height}.{tx_id}.{tx_index}.{receipt_index}.{contract_id}.{sub_id}"]
pub struct ReceiptsBurnSubject {
    pub block_height: Option<BlockHeight>,
    pub tx_id: Option<TxId>,
    pub tx_index: Option<u32>,
    pub receipt_index: Option<u32>,
    pub contract_id: Option<ContractId>,
    pub sub_id: Option<Bytes32>,
}
