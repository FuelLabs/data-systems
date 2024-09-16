use fuel_core_types::{fuel_tx::TxId, fuel_types::ContractId};
use fuel_streams_macros::subject::{IntoSubject, Subject};

use crate::types::*;

pub const OUTPUTS_WILDCARD_LIST: &[&str] = &[
    OutputsByIdSubject::WILDCARD,
    OutputsAllSubject::WILDCARD,
    OutputsCoinSubject::WILDCARD,
    OutputsContractSubject::WILDCARD,
    OutputsChangeSubject::WILDCARD,
    OutputsVariableSubject::WILDCARD,
    OutputsContractCreatedSubject::WILDCARD,
];

#[derive(Subject, Debug, Clone, Default)]
#[subject_wildcard = "outputs.>"]
#[subject_format = "outputs.{tx_id}.{index}.>"]
pub struct OutputsAllSubject {
    pub tx_id: Option<TxId>,
    pub index: Option<u16>,
}

#[derive(Subject, Debug, Clone, Default)]
#[subject_wildcard = "by_id.outputs.>"]
#[subject_format = "by_id.outputs.{id_kind}.{id_value}"]
pub struct OutputsByIdSubject {
    pub id_kind: Option<IdentifierKind>,
    pub id_value: Option<Bytes32>,
}

#[derive(Subject, Debug, Clone, Default)]
#[subject_wildcard = "outputs.>"]
#[subject_format = "outputs.coin.{tx_id}.{index}.{to}.{asset_id}"]
pub struct OutputsCoinSubject {
    pub tx_id: Option<TxId>,
    pub index: Option<u16>,
    pub to: Option<Address>,
    pub asset_id: Option<AssetId>,
}

#[derive(Subject, Debug, Clone, Default)]
#[subject_wildcard = "outputs.>"]
#[subject_format = "outputs.contract.{tx_id}.{index}.{contract_id}"]
pub struct OutputsContractSubject {
    pub tx_id: Option<TxId>,
    pub index: Option<u16>,
    pub contract_id: Option<ContractId>,
}

#[derive(Subject, Debug, Clone, Default)]
#[subject_wildcard = "outputs.>"]
#[subject_format = "outputs.change.{tx_id}.{index}.{to}.{asset_id}"]
pub struct OutputsChangeSubject {
    pub tx_id: Option<TxId>,
    pub index: Option<u16>,
    pub to: Option<Address>,
    pub asset_id: Option<AssetId>,
}

#[derive(Subject, Debug, Clone, Default)]
#[subject_wildcard = "outputs.>"]
#[subject_format = "outputs.variable.{tx_id}.{index}.{to}.{asset_id}"]
pub struct OutputsVariableSubject {
    pub tx_id: Option<TxId>,
    pub index: Option<u16>,
    pub to: Option<Address>,
    pub asset_id: Option<AssetId>,
}

#[derive(Subject, Debug, Clone, Default)]
#[subject_wildcard = "outputs.>"]
#[subject_format = "outputs.contract_created.{tx_id}.{index}.{contract_id}"]
pub struct OutputsContractCreatedSubject {
    pub tx_id: Option<TxId>,
    pub index: Option<u16>,
    pub contract_id: Option<ContractId>,
}

#[cfg(test)]
mod tests {
    use fuel_streams_macros::subject::SubjectBuildable;

    use super::*;

    #[test]
    fn test_output_subject_wildcard() {
        assert_eq!(OutputsAllSubject::WILDCARD, "outputs.>");
        assert_eq!(OutputsByIdSubject::WILDCARD, "by_id.outputs.>");
        assert_eq!(OutputsCoinSubject::WILDCARD, "outputs.>");
        assert_eq!(OutputsContractSubject::WILDCARD, "outputs.>");
        assert_eq!(OutputsChangeSubject::WILDCARD, "outputs.>");
        assert_eq!(OutputsVariableSubject::WILDCARD, "outputs.>");
        assert_eq!(OutputsContractCreatedSubject::WILDCARD, "outputs.>");
    }

    #[test]
    fn test_outputs_coin_subject_creation() {
        let coin_subject = OutputsCoinSubject::new()
            .with_tx_id(Some([0u8; 32].into()))
            .with_index(Some(0))
            .with_to(Some([0u8; 32].into()))
            .with_asset_id(Some([0u8; 32].into()));
        assert_eq!(coin_subject.to_string(), "outputs.coin.0000000000000000000000000000000000000000000000000000000000000000.0.0000000000000000000000000000000000000000000000000000000000000000.0000000000000000000000000000000000000000000000000000000000000000");
    }

    #[test]
    fn test_outputs_contract_created_subject_creation() {
        let contract_created_subject = OutputsContractCreatedSubject::new()
            .with_tx_id(Some([0u8; 32].into()))
            .with_index(Some(0))
            .with_contract_id(Some([0u8; 32].into()));
        assert_eq!(contract_created_subject.to_string(), "outputs.contract_created.0000000000000000000000000000000000000000000000000000000000000000.0.0000000000000000000000000000000000000000000000000000000000000000");
    }

    #[test]
    fn test_output_all_subject_creation() {
        let output_subject = OutputsAllSubject::new()
            .with_tx_id(Some([0u8; 32].into()))
            .with_index(Some(0));
        assert_eq!(output_subject.to_string(), "outputs.0000000000000000000000000000000000000000000000000000000000000000.0.>");
    }

    #[test]
    fn test_output_subject_coin() {
        let output_subject = OutputsCoinSubject::new()
            .with_tx_id(Some([0u8; 32].into()))
            .with_index(Some(0))
            .with_to(Some([0u8; 32].into()))
            .with_asset_id(Some([0u8; 32].into()));
        assert_eq!(output_subject.to_string(), "outputs.coin.0000000000000000000000000000000000000000000000000000000000000000.0.0000000000000000000000000000000000000000000000000000000000000000.0000000000000000000000000000000000000000000000000000000000000000");
    }

    #[test]
    fn test_output_subject_variable() {
        let output_subject = OutputsVariableSubject::new()
            .with_tx_id(Some([0u8; 32].into()))
            .with_index(Some(0))
            .with_to(Some([0u8; 32].into()))
            .with_asset_id(Some([1u8; 32].into()));
        assert_eq!(output_subject.to_string(), "outputs.variable.0000000000000000000000000000000000000000000000000000000000000000.0.0000000000000000000000000000000000000000000000000000000000000000.0101010101010101010101010101010101010101010101010101010101010101");
    }
}
