use fuel_core_types::fuel_tx::UniqueIdentifier;
use fuel_streams_macros::subject::{IntoSubject, Subject, SubjectBuildable};

use crate::{blocks::types::BlockHeight, types::*};

/// Represents a subject for publishing transactions that happen in the Fuel network.
///
/// This subject format allows for efficient querying and filtering of transactions
/// based on their height, index, ID, status, and kind.
///
/// # Examples
///
/// Creating a subject for a specific transaction:
///
/// ```
/// # use fuel_streams_core::transactions::TransactionsSubject;
/// # use fuel_streams_core::types::*;
/// # use fuel_streams_macros::subject::IntoSubject;
/// let subject = TransactionsSubject {
///     height: Some(23.into()),
///     tx_index: Some(1),
///     tx_id: Some(Bytes32::zeroed()),
///     status: Some(TransactionStatus::Success),
///     kind: Some(TransactionKind::Script),
/// };
/// assert_eq!(
///     subject.parse(),
///     "transactions.23.1.0x0000000000000000000000000000000000000000000000000000000000000000.success.script"
/// );
/// ```
///
/// All transactions wildcard:
///
/// ```
/// # use fuel_streams_core::transactions::TransactionsSubject;
/// assert_eq!(TransactionsSubject::WILDCARD, "transactions.>");
/// ```
///
/// Creating a subject query using the `wildcard` method:
///
/// ```
/// # use fuel_streams_core::transactions::TransactionsSubject;
/// # use fuel_streams_core::types::*;
/// let wildcard = TransactionsSubject::wildcard(None, None, Some(Bytes32::zeroed()), None, None);
/// assert_eq!(wildcard, "transactions.*.*.0x0000000000000000000000000000000000000000000000000000000000000000.*.*");
/// ```
///
/// Using the builder pattern:
///
/// ```
/// # use fuel_streams_core::transactions::TransactionsSubject;
/// # use fuel_streams_core::types::*;
/// # use fuel_streams_macros::subject::*;
/// let subject = TransactionsSubject::new()
///     .with_height(Some(23.into()))
///     .with_tx_index(Some(1))
///     .with_tx_id(Some(Bytes32::zeroed()))
///     .with_status(Some(TransactionStatus::Success))
///     .with_kind(Some(TransactionKind::Script));
/// assert_eq!(subject.parse(), "transactions.23.1.0x0000000000000000000000000000000000000000000000000000000000000000.success.script");
/// ```
#[derive(Subject, Debug, Clone, Default)]
#[subject_wildcard = "transactions.>"]
#[subject_format = "transactions.{height}.{tx_index}.{tx_id}.{status}.{kind}"]
pub struct TransactionsSubject {
    pub height: Option<BlockHeight>,
    pub tx_index: Option<usize>,
    pub tx_id: Option<Bytes32>,
    pub status: Option<TransactionStatus>,
    pub kind: Option<TransactionKind>,
}

impl From<&Transaction> for TransactionsSubject {
    fn from(value: &Transaction) -> Self {
        let subject = TransactionsSubject::new();
        let tx_id = value.cached_id().unwrap();
        let kind = TransactionKind::from(value.to_owned());
        subject.with_tx_id(Some(tx_id.into())).with_kind(Some(kind))
    }
}

/// Represents a NATS subject for querying transactions by their identifier in the Fuel network.
///
/// This subject format allows for efficient querying of transactions based on their identifier kind and value.
///
/// # Examples
///
/// Creating a subject for a specific transaction by ID:
///
/// ```
/// # use fuel_streams_core::transactions::TransactionsByIdSubject;
/// # use fuel_streams_core::types::*;
/// # use fuel_streams_macros::subject::IntoSubject;
/// let subject = TransactionsByIdSubject {
///     id_kind: Some(IdentifierKind::ContractID),
///     id_value: Some(Address::zeroed()),
/// };
/// assert_eq!(
///     subject.parse(),
///     "by_id.transactions.contract_id.0x0000000000000000000000000000000000000000000000000000000000000000"
/// );
/// ```
///
/// All transactions by ID wildcard:
///
/// ```
/// # use fuel_streams_core::transactions::TransactionsByIdSubject;
/// assert_eq!(TransactionsByIdSubject::WILDCARD, "by_id.transactions.>");
/// ```
///
/// Creating a subject query using the `wildcard` method:
///
/// ```
/// # use fuel_streams_core::transactions::TransactionsByIdSubject;
/// # use fuel_streams_core::types::*;
/// let wildcard = TransactionsByIdSubject::wildcard(Some(IdentifierKind::ContractID), None);
/// assert_eq!(wildcard, "by_id.transactions.contract_id.*");
/// ```
///
/// Using the builder pattern:
///
/// ```
/// # use fuel_streams_core::transactions::TransactionsByIdSubject;
/// # use fuel_streams_core::types::*;
/// # use fuel_streams_macros::subject::*;
/// let subject = TransactionsByIdSubject::new()
///     .with_id_kind(Some(IdentifierKind::ContractID))
///     .with_id_value(Some(Address::zeroed()));
/// assert_eq!(subject.parse(), "by_id.transactions.contract_id.0x0000000000000000000000000000000000000000000000000000000000000000");
/// ```
#[derive(Subject, Debug, Clone, Default)]
#[subject_wildcard = "by_id.transactions.>"]
#[subject_format = "by_id.transactions.{id_kind}.{id_value}"]
pub struct TransactionsByIdSubject {
    pub id_kind: Option<IdentifierKind>,
    pub id_value: Option<Address>,
}

#[cfg(test)]
mod test {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn transactions_subjects_from_transaction() {
        let mock_tx = MockTransaction::build();
        let subject = TransactionsSubject::from(&mock_tx);
        assert!(subject.height.is_none());
        assert!(subject.tx_index.is_none());
        assert!(subject.status.is_none());
        assert!(subject.kind.is_some());
        assert_eq!(
            subject.tx_id.unwrap(),
            mock_tx.to_owned().cached_id().unwrap().into()
        );
    }
}
