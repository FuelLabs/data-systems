use fuel_streams_macros::subject::{IntoSubject, Subject};

use super::{Bytes32, MessageId};

/// Represents a subject for utxos coins in the Fuel network.
///
/// This subject format allows for efficient querying and filtering of utxos coins
/// based on their transaction ID.
///
/// # Examples
///
/// Creating a subject for a specific utxo coin:
///
/// ```
/// # use fuel_streams_core::utxos::subjects::UtxosCoinSubject;
/// # use fuel_streams_core::types::*;
/// # use fuel_streams_macros::subject::*;
/// let subject = UtxosCoinSubject {
///     tx_id: Some(Bytes32::from([1u8; 32])),
/// };
/// assert_eq!(
///     subject.parse(),
///     "utxos.coin.0x0101010101010101010101010101010101010101010101010101010101010101"
/// );
/// ```
///
/// All utxos coin wildcard:
///
/// ```
/// # use fuel_streams_core::utxos::subjects::UtxosCoinSubject;
/// # use fuel_streams_macros::subject::*;
/// assert_eq!(UtxosCoinSubject::WILDCARD, "utxos.coin.>");
/// ```
///
/// Creating a subject query using the `wildcard` method:
///
/// ```
/// # use fuel_streams_core::utxos::subjects::UtxosCoinSubject;
/// # use fuel_streams_core::types::*;
/// # use fuel_streams_macros::subject::*;
/// let wildcard = UtxosCoinSubject::wildcard(None, Some(0), None, Some(AssetId::from([3u8; 32])));
/// assert_eq!(wildcard, "utxos.coin.*");
/// ```
///
/// Using the builder pattern:
///
/// ```
/// # use fuel_streams_core::utxos::subjects::UtxosCoinSubject;
/// # use fuel_streams_core::types::*;
/// # use fuel_streams_macros::subject::*;
/// let subject = UtxosCoinSubject::new()
///     .with_tx_id(Some(Bytes32::from([1u8; 32])));
/// assert_eq!(subject.parse(), "utxos.coin.0x0101010101010101010101010101010101010101010101010101010101010101");
/// ```
#[derive(Subject, Debug, Clone, Default)]
#[subject_wildcard = "utxos.coin.>"]
#[subject_format = "utxos.coin.{tx_id}"]
pub struct UtxosCoinSubject {
    pub tx_id: Option<Bytes32>,
}

/// Represents a subject for utxos contracts in the Fuel network.
///
/// This subject format allows for efficient querying and filtering of utxos contracts
/// based on their transaction ID.
///
/// # Examples
///
/// Creating a subject for a specific utxo contract:
///
/// ```
/// # use fuel_streams_core::utxos::subjects::UtxosContractSubject;
/// # use fuel_streams_core::types::*;
/// # use fuel_streams_macros::subject::*;
/// let subject = UtxosContractSubject {
///     tx_id: Some(Bytes32::from([1u8; 32])),
/// };
/// assert_eq!(
///     subject.parse(),
///     "utxo.contract.0x0101010101010101010101010101010101010101010101010101010101010101"
/// );
/// ```
///
/// All utxo contracts wildcard:
///
/// ```
/// # use fuel_streams_core::utxos::subjects::UtxosContractSubject;
/// # use fuel_streams_macros::subject::*;
/// assert_eq!(UtxosContractSubject::WILDCARD, "utxos.contract.>");
/// ```
///
/// Creating a subject query using the `wildcard` method:
///
/// ```
/// # use fuel_streams_core::utxos::subjects::UtxosContractSubject;
/// # use fuel_streams_core::types::*;
/// # use fuel_streams_macros::subject::*;
/// let wildcard = UtxosContractSubject::wildcard(Some(Bytes32::from([1u8; 32])));
/// assert_eq!(wildcard, "utxos.contract.0x0101010101010101010101010101010101010101010101010101010101010101");
/// ```
///
/// Using the builder pattern:
///
/// ```
/// # use fuel_streams_core::utxos::subjects::UtxosContractSubject;
/// # use fuel_streams_core::types::*;
/// # use fuel_streams_macros::subject::*;
/// let subject = UtxosContractSubject::new()
///     .with_tx_id(Some(Bytes32::from([1u8; 32])));
/// assert_eq!(subject.parse(), "utxos.contract.0x0101010101010101010101010101010101010101010101010101010101010101");
/// ```
#[derive(Subject, Debug, Clone, Default)]
#[subject_wildcard = "utxos.contract.>"]
#[subject_format = "utxos.contract.{tx_id}"]
pub struct UtxosContractSubject {
    pub tx_id: Option<Bytes32>,
}

/// Represents a subject for utxo messages in the Fuel ecosystem.
///
/// This struct is used to create and parse subjects related to utxo messages,
/// which can be used for subscribing to or publishing events about utxo messages.
///
/// # Examples
///
/// Creating and parsing a subject:
///
/// ```
/// # use fuel_streams_core::utxos::subjects::UtxosMessageSubject;
/// # use fuel_streams_core::types::*;
/// # use fuel_streams_macros::subject::*;
/// let subject = UtxosMessageSubject {
///     tx_id: Some(Bytes32::from([1u8; 32])),
/// };
/// assert_eq!(
///     subject.parse(),
///     "utxos.message.0x0101010101010101010101010101010101010101010101010101010101010101"
/// );
/// ```
///
/// All utxos messages wildcard:
///
/// ```
/// # use fuel_streams_core::utxos::subjects::UtxosMessageSubject;
/// # use fuel_streams_macros::subject::*;
/// assert_eq!(UtxosMessageSubject::WILDCARD, "utxos.message.>");
/// ```
///
/// Creating a subject query using the `wildcard` method:
///
/// ```
/// # use fuel_streams_core::utxos::subjects::UtxosMessageSubject;
/// # use fuel_streams_core::types::*;
/// # use fuel_streams_macros::subject::*;
/// let wildcard = UtxosMessageSubject::wildcard(Some(Bytes32::from([1u8; 32])));
/// assert_eq!(wildcard, "utxos.message.0x0101010101010101010101010101010101010101010101010101010101010101");
/// ```
///
/// Using the builder pattern:
///
/// ```
/// # use fuel_streams_core::utxos::subjects::UtxosMessageSubject;
/// # use fuel_streams_core::types::*;
/// # use fuel_streams_macros::subject::*;
/// let subject = UtxosMessageSubject::new()
///     .with_hash(Some(Bytes32::from([1u8; 32])));
/// assert_eq!(subject.parse(), "utxos.message.0x0101010101010101010101010101010101010101010101010101010101010101");
/// ```
#[derive(Subject, Debug, Clone, Default)]
#[subject_wildcard = "utxos.message.>"]
#[subject_format = "utxos.message.{hash}"]
pub struct UtxosMessageSubject {
    pub hash: Option<MessageId>,
}
