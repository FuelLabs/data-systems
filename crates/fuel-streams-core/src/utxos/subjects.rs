use fuel_core_types::{
    fuel_tx::input::message::compute_message_id,
    fuel_types,
};
use fuel_streams_macros::subject::{IntoSubject, Subject};

use super::{types::UtxoType, Address, Bytes32, MessageId, Nonce};

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
///     tx_id: Some(Bytes32::from([1u8; 32])),
///     index: Some(0),
///     sender: Some(Address::from([2u8; 32])),
///     recipient: Some(Address::from([3u8; 32])),
/// };
/// assert_eq!(
///     subject.parse(),
///     "utxos.0x0101010101010101010101010101010101010101010101010101010101010101.0.message.0x0202020202020202020202020202020202020202020202020202020202020202.0x0303030303030303030303030303030303030303030303030303030303030303"
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
/// let wildcard = UtxosSubject::wildcard(Some(Bytes32::from([1u8; 32])), None, None, None);
/// assert_eq!(wildcard, "utxos.0x0101010101010101010101010101010101010101010101010101010101010101.*.message.*.*");
/// ```
///
/// Using the builder pattern:
///
/// ```
/// # use fuel_streams_core::utxos::subjects::UtxosSubject;
/// # use fuel_streams_core::types::*;
/// # use fuel_streams_macros::subject::*;
/// let subject = UtxosSubject::new()
///     .with_tx_id(Some(Bytes32::from([1u8; 32])))
///     .with_index(Some(0))
///     .with_sender(Some(Address::from([2u8; 32])))
///     .with_recipient(Some(Address::from([3u8; 32])));
/// assert_eq!(subject.parse(), "utxos.0x0101010101010101010101010101010101010101010101010101010101010101.0.message.0x0202020202020202020202020202020202020202020202020202020202020202.0x0303030303030303030303030303030303030303030303030303030303030303");
/// ```

#[derive(Subject, Debug, Clone, Default)]
#[subject_wildcard = "utxos.>"]
#[subject_format = "utxos.{utxo_type}.{hash}"]
#[allow(clippy::too_many_arguments)]
pub struct UtxosSubject {
    pub hash: Option<MessageId>,
    pub tx_id: Option<Bytes32>,
    pub utxo_type: Option<UtxoType>,
    pub sender: Option<Address>,
    pub recipient: Option<Address>,
    pub nonce: Option<Nonce>,
    pub data: Option<String>, // hexified data
    pub amount: Option<u64>,
}

impl UtxosSubject {
    pub fn with_hexified_data(self, data: Option<Vec<u8>>) -> Self {
        self.with_data(data.map(hex::encode))
    }

    pub fn with_computed_hash(self) -> Self {
        let utxo_type = self.utxo_type.clone().unwrap_or_default();
        let self_clone = self.clone();
        let msg_id = self.clone().tx_id.map(|e| {
            MessageId::new(fuel_types::MessageId::new(*e.into_inner()))
        });
        match utxo_type {
            UtxoType::Message if self_clone.hash.is_none() => {
                match (
                    self.sender.as_ref(),
                    self.recipient.as_ref(),
                    self.nonce.as_ref(),
                    self.amount.as_ref(),
                    self.data.as_ref(),
                ) {
                    (
                        Some(sender),
                        Some(recipient),
                        Some(nonce),
                        Some(amount),
                        Some(data),
                    ) => {
                        let computed_hash = compute_message_id(
                            sender.as_ref(),
                            recipient.as_ref(),
                            nonce.as_ref(),
                            *amount,
                            data.as_ref(),
                        );
                        self_clone.with_hash(Some(computed_hash.into()))
                    }
                    _ => self_clone.with_hash(msg_id),
                }
            }
            _ => self_clone.with_hash(msg_id),
        }
    }
}

#[cfg(test)]
mod tests {
    use fuel_core_types::fuel_types::{Address, Bytes32};
    use fuel_streams_macros::subject::SubjectBuildable;

    use super::*;

    #[test]
    fn test_utxos_subject_wildcard() {
        assert_eq!(UtxosSubject::WILDCARD, "utxos.>");
    }

    #[test]
    fn test_utxos_message_subject_creation() {
        let utxo_subject = UtxosSubject::new()
            .with_tx_id(Some(Bytes32::zeroed().into()))
            .with_amount(Some(0))
            .with_recipient(Some(Address::zeroed().into()))
            .with_sender(Some(Address::zeroed().into()))
            .with_utxo_type(Some(UtxoType::Message))
            .with_hexified_data(Some(vec![100; 1]))
            .with_nonce(Some(Nonce::zeroed()))
            .with_computed_hash();
        assert!(utxo_subject.to_string().contains("utxos.message"));
    }

    #[test]
    fn test_utxos_coin_subject_creation() {
        let utxo_subject = UtxosSubject::new()
            .with_tx_id(Some(Bytes32::zeroed().into()))
            .with_utxo_type(Some(UtxoType::Coin))
            .with_computed_hash();
        assert_eq!(
            utxo_subject.to_string(),
            "utxos.coin.0x0000000000000000000000000000000000000000000000000000000000000000"
        );
    }

    #[test]
    fn test_utxos_contract_subject_creation() {
        let utxo_subject = UtxosSubject::new()
            .with_tx_id(Some(Bytes32::zeroed().into()))
            .with_utxo_type(Some(UtxoType::Contract))
            .with_computed_hash();
        assert_eq!(
            utxo_subject.to_string(),
            "utxos.contract.0x0000000000000000000000000000000000000000000000000000000000000000"
        );
    }
}
