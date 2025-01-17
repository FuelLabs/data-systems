use fuel_streams_macros::subject::*;
use fuel_streams_types::*;
use serde::{Deserialize, Serialize};

use crate::blocks::types::*;

#[derive(Subject, Debug, Clone, Default, Serialize, Deserialize)]
#[subject(id = "receipts_call")]
#[subject(entity = "Receipt")]
#[subject(wildcard = "receipts.call.>")]
#[subject(
    format = "receipts.call.{block_height}.{tx_id}.{tx_index}.{receipt_index}.{from}.{to}.{asset}"
)]
pub struct ReceiptsCallSubject {
    pub block_height: Option<BlockHeight>,
    pub tx_id: Option<TxId>,
    pub tx_index: Option<u32>,
    pub receipt_index: Option<u32>,
    #[subject(sql_column = "from_contract_id")]
    pub from: Option<ContractId>,
    #[subject(sql_column = "to_contract_id")]
    pub to: Option<ContractId>,
    #[subject(sql_column = "asset_id")]
    pub asset: Option<AssetId>,
}

#[derive(Subject, Debug, Clone, Default, Serialize, Deserialize)]
#[subject(id = "receipts_return")]
#[subject(entity = "Receipt")]
#[subject(wildcard = "receipts.return.>")]
#[subject(
    format = "receipts.return.{block_height}.{tx_id}.{tx_index}.{receipt_index}.{contract}"
)]
pub struct ReceiptsReturnSubject {
    pub block_height: Option<BlockHeight>,
    pub tx_id: Option<TxId>,
    pub tx_index: Option<u32>,
    pub receipt_index: Option<u32>,
    #[subject(sql_column = "contract_id")]
    pub contract: Option<ContractId>,
}

#[derive(Subject, Debug, Clone, Default, Serialize, Deserialize)]
#[subject(id = "receipts_return_data")]
#[subject(entity = "Receipt")]
#[subject(wildcard = "receipts.return_data.>")]
#[subject(
    format = "receipts.return_data.{block_height}.{tx_id}.{tx_index}.{receipt_index}.{contract}"
)]
pub struct ReceiptsReturnDataSubject {
    pub block_height: Option<BlockHeight>,
    pub tx_id: Option<TxId>,
    pub tx_index: Option<u32>,
    pub receipt_index: Option<u32>,
    #[subject(sql_column = "contract_id")]
    pub contract: Option<ContractId>,
}

#[derive(Subject, Debug, Clone, Default, Serialize, Deserialize)]
#[subject(id = "receipts_panic")]
#[subject(entity = "Receipt")]
#[subject(wildcard = "receipts.panic.>")]
#[subject(
    format = "receipts.panic.{block_height}.{tx_id}.{tx_index}.{receipt_index}.{contract}"
)]
pub struct ReceiptsPanicSubject {
    pub block_height: Option<BlockHeight>,
    pub tx_id: Option<TxId>,
    pub tx_index: Option<u32>,
    pub receipt_index: Option<u32>,
    #[subject(sql_column = "contract_id")]
    pub contract: Option<ContractId>,
}

#[derive(Subject, Debug, Clone, Default, Serialize, Deserialize)]
#[subject(id = "receipts_revert")]
#[subject(entity = "Receipt")]
#[subject(wildcard = "receipts.revert.>")]
#[subject(
    format = "receipts.revert.{block_height}.{tx_id}.{tx_index}.{receipt_index}.{contract}"
)]
pub struct ReceiptsRevertSubject {
    pub block_height: Option<BlockHeight>,
    pub tx_id: Option<TxId>,
    pub tx_index: Option<u32>,
    pub receipt_index: Option<u32>,
    #[subject(sql_column = "contract_id")]
    pub contract: Option<ContractId>,
}

#[derive(Subject, Debug, Clone, Default, Serialize, Deserialize)]
#[subject(id = "receipts_log")]
#[subject(entity = "Receipt")]
#[subject(wildcard = "receipts.log.>")]
#[subject(
    format = "receipts.log.{block_height}.{tx_id}.{tx_index}.{receipt_index}.{contract}"
)]
pub struct ReceiptsLogSubject {
    pub block_height: Option<BlockHeight>,
    pub tx_id: Option<TxId>,
    pub tx_index: Option<u32>,
    pub receipt_index: Option<u32>,
    #[subject(sql_column = "contract_id")]
    pub contract: Option<ContractId>,
}

#[derive(Subject, Debug, Clone, Default, Serialize, Deserialize)]
#[subject(id = "receipts_log_data")]
#[subject(entity = "Receipt")]
#[subject(wildcard = "receipts.log_data.>")]
#[subject(
    format = "receipts.log_data.{block_height}.{tx_id}.{tx_index}.{receipt_index}.{contract}"
)]
pub struct ReceiptsLogDataSubject {
    pub block_height: Option<BlockHeight>,
    pub tx_id: Option<TxId>,
    pub tx_index: Option<u32>,
    pub receipt_index: Option<u32>,
    #[subject(sql_column = "contract_id")]
    pub contract: Option<ContractId>,
}

#[derive(Subject, Debug, Clone, Default, Serialize, Deserialize)]
#[subject(id = "receipts_transfer")]
#[subject(entity = "Receipt")]
#[subject(wildcard = "receipts.transfer.>")]
#[subject(
    format = "receipts.transfer.{block_height}.{tx_id}.{tx_index}.{receipt_index}.{from}.{to}.{asset}"
)]
pub struct ReceiptsTransferSubject {
    pub block_height: Option<BlockHeight>,
    pub tx_id: Option<TxId>,
    pub tx_index: Option<u32>,
    pub receipt_index: Option<u32>,
    #[subject(sql_column = "from_contract_id")]
    pub from: Option<ContractId>,
    #[subject(sql_column = "to_contract_id")]
    pub to: Option<ContractId>,
    #[subject(sql_column = "asset_id")]
    pub asset: Option<AssetId>,
}

#[derive(Subject, Debug, Clone, Default, Serialize, Deserialize)]
#[subject(id = "receipts_transfer_out")]
#[subject(entity = "Receipt")]
#[subject(wildcard = "receipts.transfer_out.>")]
#[subject(
    format = "receipts.transfer_out.{block_height}.{tx_id}.{tx_index}.{receipt_index}.{from}.{to_address}.{asset}"
)]
pub struct ReceiptsTransferOutSubject {
    pub block_height: Option<BlockHeight>,
    pub tx_id: Option<TxId>,
    pub tx_index: Option<u32>,
    pub receipt_index: Option<u32>,
    #[subject(sql_column = "from_contract_id")]
    pub from: Option<ContractId>,
    #[subject(sql_column = "to_address")]
    pub to_address: Option<Address>,
    #[subject(sql_column = "asset_id")]
    pub asset: Option<AssetId>,
}

#[derive(Subject, Debug, Clone, Default, Serialize, Deserialize)]
#[subject(id = "receipts_script_result")]
#[subject(entity = "Receipt")]
#[subject(wildcard = "receipts.script_result.>")]
#[subject(
    format = "receipts.script_result.{block_height}.{tx_id}.{tx_index}.{receipt_index}"
)]
pub struct ReceiptsScriptResultSubject {
    pub block_height: Option<BlockHeight>,
    pub tx_id: Option<TxId>,
    pub tx_index: Option<u32>,
    pub receipt_index: Option<u32>,
}

#[derive(Subject, Debug, Clone, Default, Serialize, Deserialize)]
#[subject(id = "receipts_message_out")]
#[subject(entity = "Receipt")]
#[subject(wildcard = "receipts.message_out.>")]
#[subject(
    format = "receipts.message_out.{block_height}.{tx_id}.{tx_index}.{receipt_index}.{sender}.{recipient}"
)]
pub struct ReceiptsMessageOutSubject {
    pub block_height: Option<BlockHeight>,
    pub tx_id: Option<TxId>,
    pub tx_index: Option<u32>,
    pub receipt_index: Option<u32>,
    #[subject(sql_column = "sender_address")]
    pub sender: Option<Address>,
    #[subject(sql_column = "recipient_address")]
    pub recipient: Option<Address>,
}

#[derive(Subject, Debug, Clone, Default, Serialize, Deserialize)]
#[subject(id = "receipts_mint")]
#[subject(entity = "Receipt")]
#[subject(wildcard = "receipts.mint.>")]
#[subject(
    format = "receipts.mint.{block_height}.{tx_id}.{tx_index}.{receipt_index}.{contract}.{sub_id}"
)]
pub struct ReceiptsMintSubject {
    pub block_height: Option<BlockHeight>,
    pub tx_id: Option<TxId>,
    pub tx_index: Option<u32>,
    pub receipt_index: Option<u32>,
    #[subject(sql_column = "contract_id")]
    pub contract: Option<ContractId>,
    pub sub_id: Option<Bytes32>,
}

#[derive(Subject, Debug, Clone, Default, Serialize, Deserialize)]
#[subject(id = "receipts_burn")]
#[subject(entity = "Receipt")]
#[subject(wildcard = "receipts.burn.>")]
#[subject(
    format = "receipts.burn.{block_height}.{tx_id}.{tx_index}.{receipt_index}.{contract}.{sub_id}"
)]
pub struct ReceiptsBurnSubject {
    pub block_height: Option<BlockHeight>,
    pub tx_id: Option<TxId>,
    pub tx_index: Option<u32>,
    pub receipt_index: Option<u32>,
    #[subject(sql_column = "contract_id")]
    pub contract: Option<ContractId>,
    pub sub_id: Option<Bytes32>,
}
