use fuel_streams_macros::subject::{IntoSubject, Subject};

use crate::types::*;

/// Represents a subject for utxos messages in the Fuel ecosystem.
///
/// This struct is used to create and parse subjects related to utxos messages,
/// which can be used for subscribing to or publishing events about utxos messages.
///
/// # Examples
///
/// Creating and parsing a subject:
///
/// ```
/// # use fuel_streams_core::utxos::subjects::UtxosSubject;
/// # use fuel_streams_core::types::*;
/// # use fuel_streams_macros::subject::*;
/// let subject = UtxosSubject {
///     utxo_id: Some(HexString::zeroed()),
///     utxo_type: Some(UtxoType::Message),
/// };
/// assert_eq!(
///     subject.parse(),
///     "utxos.message.0x0000000000000000000000000000000000000000000000000000000000000000"
/// );
/// ```
///
/// All utxos messages wildcard:
///
/// ```
/// # use fuel_streams_core::utxos::subjects::UtxosSubject;
/// # use fuel_streams_macros::subject::*;
/// assert_eq!(UtxosSubject::WILDCARD, "utxos.>");
/// ```
///
/// Creating a subject query using the `wildcard` method:
///
/// ```
/// # use fuel_streams_core::utxos::subjects::UtxosSubject;
/// # use fuel_streams_core::types::*;
/// # use fuel_streams_macros::subject::*;
/// let wildcard = UtxosSubject::wildcard(
///     Some(HexString::zeroed()),
///     None,
/// );
/// assert_eq!(wildcard, "utxos.*.0x0000000000000000000000000000000000000000000000000000000000000000");
/// ```
///
/// Using the builder pattern:
///
/// ```
/// # use fuel_streams_core::utxos::subjects::UtxosSubject;
/// # use fuel_streams_core::types::*;
/// # use fuel_streams_macros::subject::*;
/// let subject = UtxosSubject::new()
///     .with_utxo_id(Some(HexString::zeroed()))
///     .with_utxo_type(Some(UtxoType::Message));
/// assert_eq!(subject.parse(), "utxos.message.0x0000000000000000000000000000000000000000000000000000000000000000");
/// ```

#[derive(Subject, Debug, Clone, Default)]
#[subject_wildcard = "utxos.>"]
#[subject_format = "utxos.{utxo_type}.{utxo_id}"]
pub struct UtxosSubject {
    pub utxo_id: Option<HexString>,
    pub utxo_type: Option<UtxoType>,
}

#[cfg(test)]
mod tests {
    use fuel_streams_macros::subject::SubjectBuildable;

    use super::*;

    #[test]
    fn test_utxos_subject_wildcard() {
        assert_eq!(UtxosSubject::WILDCARD, "utxos.>");
    }

    #[test]
    fn test_utxos_message_subject_creation() {
        let utxo_subject = UtxosSubject::new()
            .with_utxo_id(Some(HexString::zeroed()))
            .with_utxo_type(Some(UtxoType::Message));
        assert_eq!(
            utxo_subject.to_string(),
            "utxos.message.0x0000000000000000000000000000000000000000000000000000000000000000"
        );
    }

    #[test]
    fn test_utxos_coin_subject_creation() {
        let utxo_subject = UtxosSubject::new()
            .with_utxo_id(Some(HexString::zeroed()))
            .with_utxo_type(Some(UtxoType::Coin));
        assert_eq!(
            utxo_subject.to_string(),
            "utxos.coin.0x0000000000000000000000000000000000000000000000000000000000000000"
        );
    }

    #[test]
    fn test_utxos_contract_subject_creation() {
        let utxo_subject = UtxosSubject::new()
            .with_utxo_id(Some(HexString::zeroed()))
            .with_utxo_type(Some(UtxoType::Contract));
        assert_eq!(
            utxo_subject.to_string(),
            "utxos.contract.0x0000000000000000000000000000000000000000000000000000000000000000"
        );
    }
}
