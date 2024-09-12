use fuel_streams_macros::subject::{IntoSubject, Subject};

use super::{AssetId, Bytes32, ContractId, IdentifierKind};
use crate::prelude::Address;

/// Represents a subject for input coins in the Fuel network.
///
/// This subject format allows for efficient querying and filtering of input coins
/// based on their transaction ID, index, owner, and asset ID.
///
/// # Examples
///
/// Creating a subject for a specific input coin:
///
/// ```
/// # use fuel_streams_core::inputs::subjects::InputsCoinSubject;
/// # use fuel_streams_core::types::*;
/// # use fuel_streams_macros::subject::IntoSubject;
/// let subject = InputsCoinSubject {
///     tx_id: Some(Bytes32::from([1u8; 32])),
///     index: Some(0),
///     owner: Some(Address::from([2u8; 32])),
///     asset_id: Some(AssetId::from([3u8; 32])),
/// };
/// assert_eq!(
///     subject.parse(),
///     "inputs.0x0101010101010101010101010101010101010101010101010101010101010101.0.coin.0x0202020202020202020202020202020202020202020202020202020202020202.0x0303030303030303030303030303030303030303030303030303030303030303"
/// );
/// ```
///
/// All input coins wildcard:
///
/// ```
/// # use fuel_streams_core::inputs::subjects::InputsCoinSubject;
/// # use fuel_streams_macros::subject::IntoSubject;
/// assert_eq!(InputsCoinSubject::WILDCARD, "inputs.>");
/// ```
///
/// Creating a subject query using the `wildcard` method:
///
/// ```
/// # use fuel_streams_core::inputs::subjects::InputsCoinSubject;
/// # use fuel_streams_core::types::*;
/// # use fuel_streams_macros::subject::IntoSubject;
/// let wildcard = InputsCoinSubject::wildcard(None, Some(0), None, Some(AssetId::from([3u8; 32])));
/// assert_eq!(wildcard, "inputs.*.0.coin.*.0x0303030303030303030303030303030303030303030303030303030303030303");
/// ```
///
/// Using the builder pattern:
///
/// ```
/// # use fuel_streams_core::inputs::subjects::InputsCoinSubject;
/// # use fuel_streams_core::types::*;
/// # use fuel_streams_macros::subject::IntoSubject;
/// let subject = InputsCoinSubject::new()
///     .with_tx_id(Some(Bytes32::from([1u8; 32])))
///     .with_index(Some(0))
///     .with_owner(Some(Address::from([2u8; 32])))
///     .with_asset_id(Some(AssetId::from([3u8; 32])));
/// assert_eq!(subject.parse(), "inputs.0x0101010101010101010101010101010101010101010101010101010101010101.0.coin.0x0202020202020202020202020202020202020202020202020202020202020202.0x0303030303030303030303030303030303030303030303030303030303030303");
/// ```
#[derive(Subject, Debug, Clone, Default)]
#[subject_wildcard = "inputs.>"]
#[subject_format = "inputs.{tx_id}.{index}.coin.{owner}.{asset_id}"]
pub struct InputsCoinSubject {
    pub tx_id: Option<Bytes32>,
    pub index: Option<usize>,
    pub owner: Option<Address>,
    pub asset_id: Option<AssetId>,
}

/// Represents a subject for input contracts in the Fuel network.
///
/// This subject format allows for efficient querying and filtering of input contracts
/// based on their transaction ID, index, and contract ID.
///
/// # Examples
///
/// Creating a subject for a specific input contract:
///
/// ```
/// # use fuel_streams_core::inputs::subjects::InputsContractSubject;
/// # use fuel_streams_core::types::*;
/// # use fuel_streams_macros::subject::IntoSubject;
/// let subject = InputsContractSubject {
///     tx_id: Some(Bytes32::from([1u8; 32])),
///     index: Some(0),
///     contract_id: Some(ContractId::from([4u8; 32])),
/// };
/// assert_eq!(
///     subject.parse(),
///     "inputs.0x0101010101010101010101010101010101010101010101010101010101010101.0.contract.0x0404040404040404040404040404040404040404040404040404040404040404"
/// );
/// ```
///
/// All input contracts wildcard:
///
/// ```
/// # use fuel_streams_core::inputs::subjects::InputsContractSubject;
/// # use fuel_streams_macros::subject::IntoSubject;
/// assert_eq!(InputsContractSubject::WILDCARD, "inputs.>");
/// ```
///
/// Creating a subject query using the `wildcard` method:
///
/// ```
/// # use fuel_streams_core::inputs::subjects::InputsContractSubject;
/// # use fuel_streams_core::types::*;
/// # use fuel_streams_macros::subject::IntoSubject;
/// let wildcard = InputsContractSubject::wildcard(Some(Bytes32::from([1u8; 32])), None, None);
/// assert_eq!(wildcard, "inputs.0x0101010101010101010101010101010101010101010101010101010101010101.*.contract.*");
/// ```
///
/// Using the builder pattern:
///
/// ```
/// # use fuel_streams_core::inputs::subjects::InputsContractSubject;
/// # use fuel_streams_core::types::*;
/// # use fuel_streams_macros::subject::IntoSubject;
/// let subject = InputsContractSubject::new()
///     .with_tx_id(Some(Bytes32::from([1u8; 32])))
///     .with_index(Some(0))
///     .with_contract_id(Some(ContractId::from([4u8; 32])));
/// assert_eq!(subject.parse(), "inputs.0x0101010101010101010101010101010101010101010101010101010101010101.0.contract.0x0404040404040404040404040404040404040404040404040404040404040404");
/// ```
#[derive(Subject, Debug, Clone, Default)]
#[subject_wildcard = "inputs.>"]
#[subject_format = "inputs.{tx_id}.{index}.contract.{contract_id}"]
pub struct InputsContractSubject {
    pub tx_id: Option<Bytes32>,
    pub index: Option<usize>,
    pub contract_id: Option<ContractId>,
}

/// Represents a subject for input messages in the Fuel ecosystem.
///
/// This struct is used to create and parse subjects related to input messages,
/// which can be used for subscribing to or publishing events about input messages.
///
/// # Examples
///
/// Creating and parsing a subject:
///
/// ```
/// # use fuel_streams_core::inputs::subjects::InputsMessageSubject;
/// # use fuel_streams_core::types::*;
/// # use fuel_streams_macros::subject::IntoSubject;
/// let subject = InputsMessageSubject {
///     tx_id: Some(Bytes32::from([1u8; 32])),
///     index: Some(0),
///     sender: Some(Address::from([2u8; 32])),
///     recipient: Some(Address::from([3u8; 32])),
/// };
/// assert_eq!(
///     subject.parse(),
///     "inputs.0x0101010101010101010101010101010101010101010101010101010101010101.0.message.0x0202020202020202020202020202020202020202020202020202020202020202.0x0303030303030303030303030303030303030303030303030303030303030303"
/// );
/// ```
///
/// All input messages wildcard:
///
/// ```
/// # use fuel_streams_core::inputs::subjects::InputsMessageSubject;
/// # use fuel_streams_macros::subject::IntoSubject;
/// assert_eq!(InputsMessageSubject::WILDCARD, "inputs.>");
/// ```
///
/// Creating a subject query using the `wildcard` method:
///
/// ```
/// # use fuel_streams_core::inputs::subjects::InputsMessageSubject;
/// # use fuel_streams_core::types::*;
/// # use fuel_streams_macros::subject::IntoSubject;
/// let wildcard = InputsMessageSubject::wildcard(Some(Bytes32::from([1u8; 32])), None, None, None);
/// assert_eq!(wildcard, "inputs.0x0101010101010101010101010101010101010101010101010101010101010101.*.message.*.*");
/// ```
///
/// Using the builder pattern:
///
/// ```
/// # use fuel_streams_core::inputs::subjects::InputsMessageSubject;
/// # use fuel_streams_core::types::*;
/// # use fuel_streams_macros::subject::IntoSubject;
/// let subject = InputsMessageSubject::new()
///     .with_tx_id(Some(Bytes32::from([1u8; 32])))
///     .with_index(Some(0))
///     .with_sender(Some(Address::from([2u8; 32])))
///     .with_recipient(Some(Address::from([3u8; 32])));
/// assert_eq!(subject.parse(), "inputs.0x0101010101010101010101010101010101010101010101010101010101010101.0.message.0x0202020202020202020202020202020202020202020202020202020202020202.0x0303030303030303030303030303030303030303030303030303030303030303");
/// ```
#[derive(Subject, Debug, Clone, Default)]
#[subject_wildcard = "inputs.>"]
#[subject_format = "inputs.{tx_id}.{index}.message.{sender}.{recipient}"]
pub struct InputsMessageSubject {
    pub tx_id: Option<Bytes32>,
    pub index: Option<usize>,
    pub sender: Option<Address>,
    pub recipient: Option<Address>,
}

/// Represents a subject for querying inputs by their identifier in the Fuel ecosystem.
///
/// This struct is used to create and parse subjects related to inputs identified by
/// various types of IDs, which can be used for subscribing to or publishing events
/// about specific inputs.
///
/// # Examples
///
/// Creating and parsing a subject:
///
/// ```
/// # use fuel_streams_core::inputs::subjects::InputsByIdSubject;
/// # use fuel_streams_core::types::*;
/// # use fuel_streams_macros::subject::IntoSubject;
/// let subject = InputsByIdSubject {
///     id_kind: Some(IdentifierKind::AssetID),
///     id_value: Some([3u8; 32].into()),
/// };
/// assert_eq!(
///     subject.parse(),
///     "by_id.inputs.asset_id.0x0303030303030303030303030303030303030303030303030303030303030303"
/// );
/// ```
///
/// All inputs by ID wildcard:
///
/// ```
/// # use fuel_streams_core::inputs::subjects::InputsByIdSubject;
/// # use fuel_streams_macros::subject::IntoSubject;
/// assert_eq!(InputsByIdSubject::WILDCARD, "by_id.inputs.>");
/// ```
///
/// Creating a subject query using the `wildcard` method:
///
/// ```
/// # use fuel_streams_core::inputs::subjects::InputsByIdSubject;
/// # use fuel_streams_core::types::*;
/// # use fuel_streams_macros::subject::IntoSubject;
/// let wildcard = InputsByIdSubject::wildcard(Some(IdentifierKind::AssetID), None);
/// assert_eq!(wildcard, "by_id.inputs.asset_id.*");
/// ```
///
/// Using the builder pattern:
///
/// ```
/// # use fuel_streams_core::inputs::subjects::InputsByIdSubject;
/// # use fuel_streams_core::types::*;
/// # use fuel_streams_macros::subject::IntoSubject;
/// let subject = InputsByIdSubject::new()
///     .with_id_kind(Some(IdentifierKind::AssetID))
///     .with_id_value(Some([3u8; 32].into()));
/// assert_eq!(subject.parse(), "by_id.inputs.asset_id.0x0303030303030303030303030303030303030303030303030303030303030303");
/// ```
#[derive(Subject, Debug, Clone, Default)]
#[subject_wildcard = "by_id.inputs.>"]
#[subject_format = "by_id.inputs.{id_kind}.{id_value}"]
pub struct InputsByIdSubject {
    pub id_kind: Option<IdentifierKind>,
    pub id_value: Option<Bytes32>,
}
