use fuel_streams_subject::subject::*;
use fuel_streams_types::*;
use serde::{Deserialize, Serialize};

use super::ReceiptType;

#[derive(Subject, Debug, Clone, Default, Serialize, Deserialize)]
#[subject(id = "receipts_call")]
#[subject(entity = "Receipt")]
#[subject(query_all = "receipts.call.>")]
#[subject(custom_where = "receipt_type = 'call'")]
#[subject(
    format = "receipts.call.{block_height}.{tx_id}.{tx_index}.{receipt_index}.{from}.{to}.{asset}"
)]
pub struct ReceiptsCallSubject {
    #[subject(
        description = "The height of the block containing this call receipt"
    )]
    pub block_height: Option<BlockHeight>,
    #[subject(
        description = "The ID of the transaction containing this call receipt (32 byte string prefixed by 0x)"
    )]
    pub tx_id: Option<TxId>,
    #[subject(description = "The index of the transaction within the block")]
    pub tx_index: Option<u32>,
    #[subject(
        description = "The index of this receipt within the transaction"
    )]
    pub receipt_index: Option<u32>,
    #[subject(
        sql_column = "from_contract_id",
        description = "The contract ID that initiated the call (32 byte string prefixed by 0x)"
    )]
    pub from: Option<ContractId>,
    #[subject(
        sql_column = "to_contract_id",
        description = "The contract ID that was called (32 byte string prefixed by 0x)"
    )]
    pub to: Option<ContractId>,
    #[subject(
        sql_column = "asset_id",
        description = "The asset ID involved in the call (32 byte string prefixed by 0x)"
    )]
    pub asset: Option<AssetId>,
}

#[derive(Subject, Debug, Clone, Default, Serialize, Deserialize)]
#[subject(id = "receipts_return")]
#[subject(entity = "Receipt")]
#[subject(query_all = "receipts.return.>")]
#[subject(custom_where = "receipt_type = 'return'")]
#[subject(
    format = "receipts.return.{block_height}.{tx_id}.{tx_index}.{receipt_index}.{contract}"
)]
pub struct ReceiptsReturnSubject {
    #[subject(
        description = "The height of the block containing this return receipt"
    )]
    pub block_height: Option<BlockHeight>,
    #[subject(
        description = "The ID of the transaction containing this return receipt (32 byte string prefixed by 0x)"
    )]
    pub tx_id: Option<TxId>,
    #[subject(description = "The index of the transaction within the block")]
    pub tx_index: Option<u32>,
    #[subject(
        description = "The index of this receipt within the transaction"
    )]
    pub receipt_index: Option<u32>,
    #[subject(
        sql_column = "contract_id",
        description = "The ID of the contract that returned (32 byte string prefixed by 0x)"
    )]
    pub contract: Option<ContractId>,
}

#[derive(Subject, Debug, Clone, Default, Serialize, Deserialize)]
#[subject(id = "receipts_return_data")]
#[subject(entity = "Receipt")]
#[subject(query_all = "receipts.return_data.>")]
#[subject(custom_where = "receipt_type = 'return_data'")]
#[subject(
    format = "receipts.return_data.{block_height}.{tx_id}.{tx_index}.{receipt_index}.{contract}"
)]
pub struct ReceiptsReturnDataSubject {
    #[subject(
        description = "The height of the block containing this return data receipt"
    )]
    pub block_height: Option<BlockHeight>,
    #[subject(
        description = "The ID of the transaction containing this return data receipt (32 byte string prefixed by 0x)"
    )]
    pub tx_id: Option<TxId>,
    #[subject(description = "The index of the transaction within the block")]
    pub tx_index: Option<u32>,
    #[subject(
        description = "The index of this receipt within the transaction"
    )]
    pub receipt_index: Option<u32>,
    #[subject(
        sql_column = "contract_id",
        description = "The ID of the contract that returned data (32 byte string prefixed by 0x)"
    )]
    pub contract: Option<ContractId>,
}

#[derive(Subject, Debug, Clone, Default, Serialize, Deserialize)]
#[subject(id = "receipts_panic")]
#[subject(entity = "Receipt")]
#[subject(query_all = "receipts.panic.>")]
#[subject(custom_where = "receipt_type = 'panic'")]
#[subject(
    format = "receipts.panic.{block_height}.{tx_id}.{tx_index}.{receipt_index}.{contract}"
)]
pub struct ReceiptsPanicSubject {
    #[subject(
        description = "The height of the block containing this panic receipt"
    )]
    pub block_height: Option<BlockHeight>,
    #[subject(
        description = "The ID of the transaction containing this panic receipt (32 byte string prefixed by 0x)"
    )]
    pub tx_id: Option<TxId>,
    #[subject(description = "The index of the transaction within the block")]
    pub tx_index: Option<u32>,
    #[subject(
        description = "The index of this receipt within the transaction"
    )]
    pub receipt_index: Option<u32>,
    #[subject(
        sql_column = "contract_id",
        description = "The ID of the contract that panicked (32 byte string prefixed by 0x)"
    )]
    pub contract: Option<ContractId>,
}

#[derive(Subject, Debug, Clone, Default, Serialize, Deserialize)]
#[subject(id = "receipts_revert")]
#[subject(entity = "Receipt")]
#[subject(query_all = "receipts.revert.>")]
#[subject(custom_where = "receipt_type = 'revert'")]
#[subject(
    format = "receipts.revert.{block_height}.{tx_id}.{tx_index}.{receipt_index}.{contract}"
)]
pub struct ReceiptsRevertSubject {
    #[subject(
        description = "The height of the block containing this revert receipt"
    )]
    pub block_height: Option<BlockHeight>,
    #[subject(
        description = "The ID of the transaction containing this revert receipt (32 byte string prefixed by 0x)"
    )]
    pub tx_id: Option<TxId>,
    #[subject(description = "The index of the transaction within the block")]
    pub tx_index: Option<u32>,
    #[subject(
        description = "The index of this receipt within the transaction"
    )]
    pub receipt_index: Option<u32>,
    #[subject(
        sql_column = "contract_id",
        description = "The ID of the contract that reverted (32 byte string prefixed by 0x)"
    )]
    pub contract: Option<ContractId>,
}

#[derive(Subject, Debug, Clone, Default, Serialize, Deserialize)]
#[subject(id = "receipts_log")]
#[subject(entity = "Receipt")]
#[subject(query_all = "receipts.log.>")]
#[subject(custom_where = "receipt_type = 'log'")]
#[subject(
    format = "receipts.log.{block_height}.{tx_id}.{tx_index}.{receipt_index}.{contract}"
)]
pub struct ReceiptsLogSubject {
    #[subject(
        description = "The height of the block containing this log receipt"
    )]
    pub block_height: Option<BlockHeight>,
    #[subject(
        description = "The ID of the transaction containing this log receipt (32 byte string prefixed by 0x)"
    )]
    pub tx_id: Option<TxId>,
    #[subject(description = "The index of the transaction within the block")]
    pub tx_index: Option<u32>,
    #[subject(
        description = "The index of this receipt within the transaction"
    )]
    pub receipt_index: Option<u32>,
    #[subject(
        sql_column = "contract_id",
        description = "The ID of the contract that emitted the log (32 byte string prefixed by 0x)"
    )]
    pub contract: Option<ContractId>,
}

#[derive(Subject, Debug, Clone, Default, Serialize, Deserialize)]
#[subject(id = "receipts_log_data")]
#[subject(entity = "Receipt")]
#[subject(query_all = "receipts.log_data.>")]
#[subject(custom_where = "receipt_type = 'log_data'")]
#[subject(
    format = "receipts.log_data.{block_height}.{tx_id}.{tx_index}.{receipt_index}.{contract}"
)]
pub struct ReceiptsLogDataSubject {
    #[subject(
        description = "The height of the block containing this log data receipt"
    )]
    pub block_height: Option<BlockHeight>,
    #[subject(
        description = "The ID of the transaction containing this log data receipt (32 byte string prefixed by 0x)"
    )]
    pub tx_id: Option<TxId>,
    #[subject(description = "The index of the transaction within the block")]
    pub tx_index: Option<u32>,
    #[subject(
        description = "The index of this receipt within the transaction"
    )]
    pub receipt_index: Option<u32>,
    #[subject(
        sql_column = "contract_id",
        description = "The ID of the contract that emitted the log data (32 byte string prefixed by 0x)"
    )]
    pub contract: Option<ContractId>,
}

#[derive(Subject, Debug, Clone, Default, Serialize, Deserialize)]
#[subject(id = "receipts_transfer")]
#[subject(entity = "Receipt")]
#[subject(query_all = "receipts.transfer.>")]
#[subject(custom_where = "receipt_type = 'transfer'")]
#[subject(
    format = "receipts.transfer.{block_height}.{tx_id}.{tx_index}.{receipt_index}.{from}.{to}.{asset}"
)]
pub struct ReceiptsTransferSubject {
    #[subject(
        description = "The height of the block containing this transfer receipt"
    )]
    pub block_height: Option<BlockHeight>,
    #[subject(
        description = "The ID of the transaction containing this transfer receipt (32 byte string prefixed by 0x)"
    )]
    pub tx_id: Option<TxId>,
    #[subject(description = "The index of the transaction within the block")]
    pub tx_index: Option<u32>,
    #[subject(
        description = "The index of this receipt within the transaction"
    )]
    pub receipt_index: Option<u32>,
    #[subject(
        sql_column = "from_contract_id",
        description = "The contract ID that initiated the transfer (32 byte string prefixed by 0x)"
    )]
    pub from: Option<ContractId>,
    #[subject(
        sql_column = "to_contract_id",
        description = "The contract ID that received the transfer (32 byte string prefixed by 0x)"
    )]
    pub to: Option<ContractId>,
    #[subject(
        sql_column = "asset_id",
        description = "The asset ID being transferred (32 byte string prefixed by 0x)"
    )]
    pub asset: Option<AssetId>,
}

#[derive(Subject, Debug, Clone, Default, Serialize, Deserialize)]
#[subject(id = "receipts_transfer_out")]
#[subject(entity = "Receipt")]
#[subject(query_all = "receipts.transfer_out.>")]
#[subject(custom_where = "receipt_type = 'transfer_out'")]
#[subject(
    format = "receipts.transfer_out.{block_height}.{tx_id}.{tx_index}.{receipt_index}.{from}.{to_address}.{asset}"
)]
pub struct ReceiptsTransferOutSubject {
    #[subject(
        description = "The height of the block containing this transfer out receipt"
    )]
    pub block_height: Option<BlockHeight>,
    #[subject(
        description = "The ID of the transaction containing this transfer out receipt (32 byte string prefixed by 0x)"
    )]
    pub tx_id: Option<TxId>,
    #[subject(description = "The index of the transaction within the block")]
    pub tx_index: Option<u32>,
    #[subject(
        description = "The index of this receipt within the transaction"
    )]
    pub receipt_index: Option<u32>,
    #[subject(
        sql_column = "from_contract_id",
        description = "The contract ID that initiated the transfer out (32 byte string prefixed by 0x)"
    )]
    pub from: Option<ContractId>,
    #[subject(
        sql_column = "to_address",
        description = "The address that received the transfer (32 byte string prefixed by 0x)"
    )]
    pub to_address: Option<Address>,
    #[subject(
        sql_column = "asset_id",
        description = "The asset ID being transferred (32 byte string prefixed by 0x)"
    )]
    pub asset: Option<AssetId>,
}

#[derive(Subject, Debug, Clone, Default, Serialize, Deserialize)]
#[subject(id = "receipts_script_result")]
#[subject(entity = "Receipt")]
#[subject(query_all = "receipts.script_result.>")]
#[subject(custom_where = "receipt_type = 'script_result'")]
#[subject(
    format = "receipts.script_result.{block_height}.{tx_id}.{tx_index}.{receipt_index}"
)]
pub struct ReceiptsScriptResultSubject {
    #[subject(
        description = "The height of the block containing this script result receipt"
    )]
    pub block_height: Option<BlockHeight>,
    #[subject(
        description = "The ID of the transaction containing this script result receipt (32 byte string prefixed by 0x)"
    )]
    pub tx_id: Option<TxId>,
    #[subject(description = "The index of the transaction within the block")]
    pub tx_index: Option<u32>,
    #[subject(
        description = "The index of this receipt within the transaction"
    )]
    pub receipt_index: Option<u32>,
}

#[derive(Subject, Debug, Clone, Default, Serialize, Deserialize)]
#[subject(id = "receipts_message_out")]
#[subject(entity = "Receipt")]
#[subject(query_all = "receipts.message_out.>")]
#[subject(custom_where = "receipt_type = 'message_out'")]
#[subject(
    format = "receipts.message_out.{block_height}.{tx_id}.{tx_index}.{receipt_index}.{sender}.{recipient}"
)]
pub struct ReceiptsMessageOutSubject {
    #[subject(
        description = "The height of the block containing this message out receipt"
    )]
    pub block_height: Option<BlockHeight>,
    #[subject(
        description = "The ID of the transaction containing this message out receipt (32 byte string prefixed by 0x)"
    )]
    pub tx_id: Option<TxId>,
    #[subject(description = "The index of the transaction within the block")]
    pub tx_index: Option<u32>,
    #[subject(
        description = "The index of this receipt within the transaction"
    )]
    pub receipt_index: Option<u32>,
    #[subject(
        sql_column = "sender_address",
        description = "The address that sent the message (32 byte string prefixed by 0x)"
    )]
    pub sender: Option<Address>,
    #[subject(
        sql_column = "recipient_address",
        description = "The address that will receive the message (32 byte string prefixed by 0x)"
    )]
    pub recipient: Option<Address>,
}

#[derive(Subject, Debug, Clone, Default, Serialize, Deserialize)]
#[subject(id = "receipts_mint")]
#[subject(entity = "Receipt")]
#[subject(query_all = "receipts.mint.>")]
#[subject(custom_where = "receipt_type = 'mint'")]
#[subject(
    format = "receipts.mint.{block_height}.{tx_id}.{tx_index}.{receipt_index}.{contract}.{sub_id}"
)]
pub struct ReceiptsMintSubject {
    #[subject(
        description = "The height of the block containing this mint receipt"
    )]
    pub block_height: Option<BlockHeight>,
    #[subject(
        description = "The ID of the transaction containing this mint receipt (32 byte string prefixed by 0x)"
    )]
    pub tx_id: Option<TxId>,
    #[subject(description = "The index of the transaction within the block")]
    pub tx_index: Option<u32>,
    #[subject(
        description = "The index of this receipt within the transaction"
    )]
    pub receipt_index: Option<u32>,
    #[subject(
        sql_column = "contract_id",
        description = "The ID of the contract that performed the mint (32 byte string prefixed by 0x)"
    )]
    pub contract: Option<ContractId>,
    #[subject(
        description = "The sub identifier of the minted asset (32 byte string prefixed by 0x)"
    )]
    pub sub_id: Option<Bytes32>,
}

#[derive(Subject, Debug, Clone, Default, Serialize, Deserialize)]
#[subject(id = "receipts_burn")]
#[subject(entity = "Receipt")]
#[subject(query_all = "receipts.burn.>")]
#[subject(custom_where = "receipt_type = 'burn'")]
#[subject(
    format = "receipts.burn.{block_height}.{tx_id}.{tx_index}.{receipt_index}.{contract}.{sub_id}"
)]
pub struct ReceiptsBurnSubject {
    #[subject(
        description = "The height of the block containing this burn receipt"
    )]
    pub block_height: Option<BlockHeight>,
    #[subject(
        description = "The ID of the transaction containing this burn receipt (32 byte string prefixed by 0x)"
    )]
    pub tx_id: Option<TxId>,
    #[subject(description = "The index of the transaction within the block")]
    pub tx_index: Option<u32>,
    #[subject(
        description = "The index of this receipt within the transaction"
    )]
    pub receipt_index: Option<u32>,
    #[subject(
        sql_column = "contract_id",
        description = "The ID of the contract that performed the burn (32 byte string prefixed by 0x)"
    )]
    pub contract: Option<ContractId>,
    #[subject(
        description = "The sub identifier of the burned asset (32 byte string prefixed by 0x)"
    )]
    pub sub_id: Option<Bytes32>,
}

// This subject is used just for query purpose, not for inserting as key
#[derive(Subject, Debug, Clone, Default, Serialize, Deserialize)]
#[subject(id = "receipts")]
#[subject(entity = "Receipt")]
#[subject(query_all = "receipts.>")]
#[subject(
    format = "receipts.{receipt_type}.{block_height}.{tx_id}.{tx_index}.{receipt_index}"
)]
pub struct ReceiptsSubject {
    #[subject(description = "The type of receipt")]
    pub receipt_type: Option<ReceiptType>,
    #[subject(description = "The height of the block containing this receipt")]
    pub block_height: Option<BlockHeight>,
    #[subject(
        description = "The ID of the transaction containing this receipt (32 byte string prefixed by 0x)"
    )]
    pub tx_id: Option<TxId>,
    #[subject(description = "The index of the transaction within the block")]
    pub tx_index: Option<u32>,
    #[subject(
        description = "The index of this receipt within the transaction"
    )]
    pub receipt_index: Option<u32>,
}
