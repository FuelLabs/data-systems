use fuel_streams_macros::subject::{IntoSubject, Subject};

use crate::types::*;

#[derive(Subject, Debug, Clone, Default)]
#[subject_wildcard = "by_id.receipts.>"]
#[subject_format = "by_id.receipts.{id_kind}.{id_value}"]
pub struct ReceiptsByIdSubject {
    pub id_kind: Option<IdentifierKind>,
    pub id_value: Option<Bytes32>,
}

#[derive(Subject, Debug, Clone, Default)]
#[subject_wildcard = "receipts.>"]
#[subject_format = "receipts.{tx_id}.{index}.call.{from}.{to}.{asset_id}"]
pub struct ReceiptsCallSubject {
    pub tx_id: Option<Bytes32>,
    pub index: Option<usize>,
    pub from: Option<ContractId>,
    pub to: Option<ContractId>,
    pub asset_id: Option<AssetId>,
}

#[derive(Subject, Debug, Clone, Default)]
#[subject_wildcard = "receipts.>"]
#[subject_format = "receipts.{tx_id}.{index}.return.{id}"]
pub struct ReceiptsReturnSubject {
    pub tx_id: Option<Bytes32>,
    pub index: Option<usize>,
    pub id: Option<ContractId>,
}

#[derive(Subject, Debug, Clone, Default)]
#[subject_wildcard = "receipts.>"]
#[subject_format = "receipts.{tx_id}.{index}.return_data.{id}"]
pub struct ReceiptsReturnDataSubject {
    pub tx_id: Option<Bytes32>,
    pub index: Option<usize>,
    pub id: Option<ContractId>,
}

#[derive(Subject, Debug, Clone, Default)]
#[subject_wildcard = "receipts.>"]
#[subject_format = "receipts.{tx_id}.{index}.panic.{id}"]
pub struct ReceiptsPanicSubject {
    pub tx_id: Option<Bytes32>,
    pub index: Option<usize>,
    pub id: Option<ContractId>,
}

#[derive(Subject, Debug, Clone, Default)]
#[subject_wildcard = "receipts.>"]
#[subject_format = "receipts.{tx_id}.{index}.revert.{id}"]
pub struct ReceiptsRevertSubject {
    pub tx_id: Option<Bytes32>,
    pub index: Option<usize>,
    pub id: Option<ContractId>,
}

#[derive(Subject, Debug, Clone, Default)]
#[subject_wildcard = "receipts.>"]
#[subject_format = "receipts.{tx_id}.{index}.log.{id}"]
pub struct ReceiptsLogSubject {
    pub tx_id: Option<Bytes32>,
    pub index: Option<usize>,
    pub id: Option<ContractId>,
}

#[derive(Subject, Debug, Clone, Default)]
#[subject_wildcard = "receipts.>"]
#[subject_format = "receipts.{tx_id}.{index}.log_data.{id}"]
pub struct ReceiptsLogDataSubject {
    pub tx_id: Option<Bytes32>,
    pub index: Option<usize>,
    pub id: Option<ContractId>,
}

#[derive(Subject, Debug, Clone, Default)]
#[subject_wildcard = "receipts.>"]
#[subject_format = "receipts.{tx_id}.{index}.transfer.{from}.{to}.{asset_id}"]
pub struct ReceiptsTransferSubject {
    pub tx_id: Option<Bytes32>,
    pub index: Option<usize>,
    pub from: Option<ContractId>,
    pub to: Option<ContractId>,
    pub asset_id: Option<AssetId>,
}

#[derive(Subject, Debug, Clone, Default)]
#[subject_wildcard = "receipts.>"]
#[subject_format = "receipts.{tx_id}.{index}.transfer_out.{from}.{to}.{asset_id}"]
pub struct ReceiptsTransferOutSubject {
    pub tx_id: Option<Bytes32>,
    pub index: Option<usize>,
    pub from: Option<ContractId>,
    pub to: Option<Address>,
    pub asset_id: Option<AssetId>,
}

#[derive(Subject, Debug, Clone, Default)]
#[subject_wildcard = "receipts.>"]
#[subject_format = "receipts.{tx_id}.{index}.script_result"]
pub struct ReceiptsScriptResultSubject {
    pub tx_id: Option<Bytes32>,
    pub index: Option<usize>,
}

#[derive(Subject, Debug, Clone, Default)]
#[subject_wildcard = "receipts.>"]
#[subject_format = "receipts.{tx_id}.{index}.message_out.{sender}.{recipient}"]
pub struct ReceiptsMessageOutSubject {
    pub tx_id: Option<Bytes32>,
    pub index: Option<usize>,
    pub sender: Option<Address>,
    pub recipient: Option<Address>,
}

#[derive(Subject, Debug, Clone, Default)]
#[subject_wildcard = "receipts.>"]
#[subject_format = "receipts.{tx_id}.{index}.mint.{contract_id}.{sub_id}"]
pub struct ReceiptsMintSubject {
    pub tx_id: Option<Bytes32>,
    pub index: Option<usize>,
    pub contract_id: Option<ContractId>,
    pub sub_id: Option<Bytes32>,
}

#[derive(Subject, Debug, Clone, Default)]
#[subject_wildcard = "receipts.>"]
#[subject_format = "receipts.{tx_id}.{index}.burn.{contract_id}.{sub_id}"]
pub struct ReceiptsBurnSubject {
    pub tx_id: Option<Bytes32>,
    pub index: Option<usize>,
    pub contract_id: Option<ContractId>,
    pub sub_id: Option<Bytes32>,
}
