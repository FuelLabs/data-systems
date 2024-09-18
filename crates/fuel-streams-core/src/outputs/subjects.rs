use fuel_core_types::fuel_types::{Address, Bytes32, ContractId};
use fuel_streams_macros::subject::{IntoSubject, Subject};

use crate::types::*;

/// Represents the NATS subject for all outputs.
///
/// This subject format allows for querying all outputs, optionally filtered by
/// transaction ID and index.
///
/// # Examples
///
/// **Creating a subject for all outputs of a specific transaction:**
///
/// ```
/// use fuel_streams_core::outputs::subjects::OutputsAllSubject;
/// use fuel_streams_core::types::*;
/// use fuel_streams_macros::subject::SubjectBuildable;
///
/// let subject = OutputsAllSubject::new()
///     .with_tx_id(Some(Bytes32::zeroed()))
///     .with_index(Some(0));
/// assert_eq!(
///     subject.to_string(),
///     "outputs.0x0000000000000000000000000000000000000000000000000000000000000000.0.>"
/// );
/// ```
#[derive(Subject, Debug, Clone, Default)]
#[subject_wildcard = "outputs.>"]
#[subject_format = "outputs.{tx_id}.{index}.>"]
pub struct OutputsAllSubject {
    pub tx_id: Option<Bytes32>,
    pub index: Option<u16>,
}

/// Represents the NATS subject for outputs by ID.
///
/// This subject format allows for querying outputs based on their ID kind and value.
///
/// # Examples
///
/// **Creating a subject for outputs by ID:**
///
/// ```
/// use fuel_streams_core::outputs::subjects::OutputsByIdSubject;
/// use fuel_streams_core::types::*;
/// use fuel_streams_macros::subject::SubjectBuildable;
///
/// let subject = OutputsByIdSubject::new()
///     .with_id_kind(Some(IdentifierKind::Address))
///     .with_id_value(Some(Bytes32::zeroed()));
/// assert_eq!(
///     subject.to_string(),
///     "by_id.outputs.address.0x0000000000000000000000000000000000000000000000000000000000000000"
/// );
/// ```
#[derive(Subject, Debug, Clone, Default)]
#[subject_wildcard = "by_id.outputs.>"]
#[subject_format = "by_id.outputs.{id_kind}.{id_value}"]
pub struct OutputsByIdSubject {
    pub id_kind: Option<IdentifierKind>,
    pub id_value: Option<Bytes32>,
}

/// Represents the NATS subject for coin outputs.
///
/// This subject format allows for querying coin outputs based on transaction ID,
/// index, recipient address (`to`), and asset ID.
///
/// # Examples
///
/// **Creating a subject for a specific coin output:**
///
/// ```
/// use fuel_streams_core::outputs::subjects::OutputsCoinSubject;
/// use fuel_streams_core::types::*;
/// use fuel_streams_macros::subject::SubjectBuildable;
///
/// let subject = OutputsCoinSubject::new()
///     .with_tx_id(Some(Bytes32::zeroed()))
///     .with_index(Some(0))
///     .with_to(Some(Address::zeroed()))
///     .with_asset_id(Some(AssetId::zeroed()));
/// assert_eq!(
///     subject.to_string(),
///     "outputs.coin.0x0000000000000000000000000000000000000000000000000000000000000000.0.0x0000000000000000000000000000000000000000000000000000000000000000.0x0000000000000000000000000000000000000000000000000000000000000000"
/// );
/// ```
#[derive(Subject, Debug, Clone, Default)]
#[subject_wildcard = "outputs.>"]
#[subject_format = "outputs.coin.{tx_id}.{index}.{to}.{asset_id}"]
pub struct OutputsCoinSubject {
    pub tx_id: Option<Bytes32>,
    pub index: Option<u16>,
    pub to: Option<Address>,
    pub asset_id: Option<AssetId>,
}

/// Represents the NATS subject for contract outputs.
///
/// This subject format allows for querying contract outputs based on
/// transaction ID, index, and contract ID.
///
/// # Examples
///
/// **Creating a subject for a specific contract output:**
///
/// ```
/// use fuel_streams_core::outputs::subjects::OutputsContractSubject;
/// use fuel_streams_core::types::*;
/// use fuel_streams_macros::subject::SubjectBuildable;
///
/// let subject = OutputsContractSubject::new()
///     .with_tx_id(Some(Bytes32::zeroed()))
///     .with_index(Some(0))
///     .with_contract_id(Some(ContractId::zeroed()));
/// assert_eq!(
///     subject.to_string(),
///     "outputs.contract.0x0000000000000000000000000000000000000000000000000000000000000000.0.0x0000000000000000000000000000000000000000000000000000000000000000"
/// );
/// ```
#[derive(Subject, Debug, Clone, Default)]
#[subject_wildcard = "outputs.>"]
#[subject_format = "outputs.contract.{tx_id}.{index}.{contract_id}"]
pub struct OutputsContractSubject {
    pub tx_id: Option<Bytes32>,
    pub index: Option<u16>,
    pub contract_id: Option<ContractId>,
}

/// Represents the NATS subject for change outputs.
///
/// This subject format allows for querying change outputs based on transaction ID,
/// index, recipient address (`to`), and asset ID.
///
/// # Examples
///
/// **Creating a subject for a specific change output:**
///
/// ```
/// use fuel_streams_core::outputs::subjects::OutputsChangeSubject;
/// use fuel_streams_core::types::*;
/// use fuel_streams_macros::subject::SubjectBuildable;
///
/// let subject = OutputsChangeSubject::new()
///     .with_tx_id(Some(Bytes32::zeroed()))
///     .with_index(Some(0))
///     .with_to(Some(Address::zeroed()))
///     .with_asset_id(Some(AssetId::zeroed()));
/// assert_eq!(
///     subject.to_string(),
///     "outputs.change.0x0000000000000000000000000000000000000000000000000000000000000000.0.0x0000000000000000000000000000000000000000000000000000000000000000.0x0000000000000000000000000000000000000000000000000000000000000000"
/// );
/// ```
#[derive(Subject, Debug, Clone, Default)]
#[subject_wildcard = "outputs.>"]
#[subject_format = "outputs.change.{tx_id}.{index}.{to}.{asset_id}"]
pub struct OutputsChangeSubject {
    pub tx_id: Option<Bytes32>,
    pub index: Option<u16>,
    pub to: Option<Address>,
    pub asset_id: Option<AssetId>,
}

/// Represents the NATS subject for variable outputs.
///
/// This subject format allows for querying variable outputs based on transaction
/// ID, index, recipient address (`to`), and asset ID.
///
/// # Examples
///
/// **Creating a subject for a specific variable output:**
///
/// ```
/// use fuel_streams_core::outputs::subjects::OutputsVariableSubject;
/// use fuel_streams_core::types::*;
/// use fuel_streams_macros::subject::SubjectBuildable;
///
/// let subject = OutputsVariableSubject::new()
///     .with_tx_id(Some(Bytes32::zeroed()))
///     .with_index(Some(0))
///     .with_to(Some(Address::zeroed()))
///     .with_asset_id(Some(Bytes32::from([1u8; 32])));
/// assert_eq!(
///     subject.to_string(),
///     "outputs.variable.0x0000000000000000000000000000000000000000000000000000000000000000.0.0x0000000000000000000000000000000000000000000000000000000000000000.0x0101010101010101010101010101010101010101010101010101010101010101"
/// );
/// ```
#[derive(Subject, Debug, Clone, Default)]
#[subject_wildcard = "outputs.>"]
#[subject_format = "outputs.variable.{tx_id}.{index}.{to}.{asset_id}"]
pub struct OutputsVariableSubject {
    pub tx_id: Option<Bytes32>,
    pub index: Option<u16>,
    pub to: Option<Address>,
    pub asset_id: Option<AssetId>,
}

/// Represents the NATS subject for contract created outputs.
///
/// This subject format allows for querying contract creation outputs based on
/// transaction ID, index, and contract ID.
///
/// # Examples
///
/// **Creating a subject for a specific contract creation output:**
///
/// ```
/// use fuel_streams_core::outputs::subjects::OutputsContractCreatedSubject;
/// use fuel_streams_core::types::*;
/// use fuel_streams_macros::subject::SubjectBuildable;
///
/// let subject = OutputsContractCreatedSubject::new()
///     .with_tx_id(Some(Bytes32::zeroed()))
///     .with_index(Some(0))
///     .with_contract_id(Some(ContractId::zeroed()));
/// assert_eq!(
///     subject.to_string(),
///     "outputs.contract_created.0x0000000000000000000000000000000000000000000000000000000000000000.0.0x0000000000000000000000000000000000000000000000000000000000000000"
/// );
/// ```
#[derive(Subject, Debug, Clone, Default)]
#[subject_wildcard = "outputs.>"]
#[subject_format = "outputs.contract_created.{tx_id}.{index}.{contract_id}"]
pub struct OutputsContractCreatedSubject {
    pub tx_id: Option<Bytes32>,
    pub index: Option<u16>,
    pub contract_id: Option<ContractId>,
}

#[cfg(test)]
mod tests {
    use fuel_core_types::fuel_types::{Address, Bytes32};
    use fuel_streams_core::types::*;
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
            .with_tx_id(Some(Bytes32::zeroed()))
            .with_index(Some(0))
            .with_to(Some(Address::zeroed()))
            .with_asset_id(Some(AssetId::zeroed()));
        assert_eq!(
            coin_subject.to_string(),
            "outputs.coin.0x0000000000000000000000000000000000000000000000000000000000000000.0.0x0000000000000000000000000000000000000000000000000000000000000000.0x0000000000000000000000000000000000000000000000000000000000000000"
        );
    }

    #[test]
    fn test_outputs_contract_created_subject_creation() {
        let contract_created_subject = OutputsContractCreatedSubject::new()
            .with_tx_id(Some(Bytes32::zeroed()))
            .with_index(Some(0))
            .with_contract_id(Some(ContractId::zeroed()));
        assert_eq!(
            contract_created_subject.to_string(),
            "outputs.contract_created.0x0000000000000000000000000000000000000000000000000000000000000000.0.0x0000000000000000000000000000000000000000000000000000000000000000"
        );
    }

    #[test]
    fn test_output_all_subject_creation() {
        let output_subject = OutputsAllSubject::new()
            .with_tx_id(Some(Bytes32::zeroed()))
            .with_index(Some(0));
        assert_eq!(
            output_subject.to_string(),
            "outputs.0x0000000000000000000000000000000000000000000000000000000000000000.0.>"
        );
    }

    #[test]
    fn test_output_subject_coin() {
        let output_subject = OutputsCoinSubject::new()
            .with_tx_id(Some(Bytes32::zeroed()))
            .with_index(Some(0))
            .with_to(Some(Address::zeroed()))
            .with_asset_id(Some(AssetId::zeroed()));
        assert_eq!(
            output_subject.to_string(),
            "outputs.coin.0x0000000000000000000000000000000000000000000000000000000000000000.0.0x0000000000000000000000000000000000000000000000000000000000000000.0x0000000000000000000000000000000000000000000000000000000000000000"
        );
    }

    #[test]
    fn test_output_subject_variable() {
        let output_subject = OutputsVariableSubject::new()
            .with_tx_id(Some(Bytes32::zeroed()))
            .with_index(Some(0))
            .with_to(Some(Address::zeroed()))
            .with_asset_id(Some(Bytes32::from([1u8; 32])));
        assert_eq!(
            output_subject.to_string(),
            "outputs.variable.0x0000000000000000000000000000000000000000000000000000000000000000.0.0x0000000000000000000000000000000000000000000000000000000000000000.0x0101010101010101010101010101010101010101010101010101010101010101"
        );
    }
}
