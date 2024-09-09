use fuel_core_types::fuel_tx::AssetId;
use fuel_streams_macros::subject::{IntoSubject, Subject};

use super::{Bytes32, ContractId, IdentifierKind};
use crate::prelude::Address;

#[derive(Subject, Debug, Clone, Default)]
#[subject_wildcard = "inputs.>"]
#[subject_format = "inputs.{tx_id}.{index}.coin.{owner}.{asset_id}"]
pub struct InputsCoinSubject {
    pub tx_id: Option<Bytes32>,
    pub index: Option<usize>,
    pub owner: Option<Address>,
    pub asset_id: Option<AssetId>,
}

#[derive(Subject, Debug, Clone, Default)]
#[subject_wildcard = "inputs.>"]
#[subject_format = "inputs.{tx_id}.{index}.contract.{contract_id}"]
pub struct InputsContractSubject {
    pub tx_id: Option<Bytes32>,
    pub index: Option<usize>,
    pub contract_id: Option<ContractId>,
}

#[derive(Subject, Debug, Clone, Default)]
#[subject_wildcard = "inputs.>"]
#[subject_format = "inputs.{tx_id}.{index}.message.{sender}.{recipient}"]
pub struct InputsMessageSubject {
    pub tx_id: Option<Bytes32>,
    pub index: Option<usize>,
    pub sender: Option<Address>,
    pub recipient: Option<Address>,
}

#[derive(Subject, Debug, Clone, Default)]
#[subject_wildcard = "by_id.inputs.>"]
#[subject_format = "by_id.inputs.{id_kind}.{id_value}"]
pub struct InputsByIdSubject {
    pub id_kind: Option<IdentifierKind>,
    pub id_value: Option<Bytes32>,
}
