use fuel_streams_macros::subject::*;
use fuel_streams_types::*;

/// Represents a subject for querying transactions by their identifier in the Fuel ecosystem.
///
/// This struct is used to create and parse subjects related to transactions identified by
/// various types of IDs, which can be used for subscribing to or publishing events
/// about specific transactions.
///
/// # Examples
///
/// Creating and parsing a subject:
///
/// ```
/// # use fuel_streams_core::subjects::*;
/// # use fuel_streams_core::prelude::*;
/// # use fuel_streams_macros::subject::*;
/// let subject = TransactionsByIdSubject {
///     tx_id: Some([1u8; 32].into()),
///     index: Some(0),
///     id_kind: Some(IdentifierKind::ContractID),
///     id_value: Some([2u8; 32].into()),
/// };
/// assert_eq!(
///     subject.parse(),
///     "by_id.transactions.0x0101010101010101010101010101010101010101010101010101010101010101.0.contract_id.0x0202020202020202020202020202020202020202020202020202020202020202"
/// );
/// ```
///
/// All transactions by ID wildcard:
///
/// ```
/// # use fuel_streams_core::subjects::*;
/// # use fuel_streams_macros::subject::*;
/// assert_eq!(TransactionsByIdSubject::WILDCARD, "by_id.transactions.>");
/// ```
///
/// Creating a subject query using the `wildcard` method:
///
/// ```
/// # use fuel_streams_core::subjects::*;
/// # use fuel_streams_core::prelude::*;
/// # use fuel_streams_macros::subject::*;
/// let wildcard = TransactionsByIdSubject::wildcard(Some([1u8; 32].into()), Some(0), Some(IdentifierKind::ContractID), None);
/// assert_eq!(wildcard, "by_id.transactions.0x0101010101010101010101010101010101010101010101010101010101010101.0.contract_id.*");
/// ```
///
/// Using the builder pattern:
///
/// ```
/// # use fuel_streams_core::subjects::*;
/// # use fuel_streams_core::prelude::*;
/// # use fuel_streams_macros::subject::*;
/// let subject = TransactionsByIdSubject::new()
///     .with_tx_id(Some([1u8; 32].into()))
///     .with_index(Some(0))
///     .with_id_kind(Some(IdentifierKind::ContractID))
///     .with_id_value(Some([2u8; 32].into()));
/// assert_eq!(subject.parse(), "by_id.transactions.0x0101010101010101010101010101010101010101010101010101010101010101.0.contract_id.0x0202020202020202020202020202020202020202020202020202020202020202");
/// ```
#[derive(Subject, Debug, Clone, Default)]
#[subject_wildcard = "by_id.transactions.>"]
#[subject_format = "by_id.transactions.{tx_id}.{index}.{id_kind}.{id_value}"]
pub struct TransactionsByIdSubject {
    pub tx_id: Option<Bytes32>,
    pub index: Option<u8>,
    pub id_kind: Option<IdentifierKind>,
    pub id_value: Option<Bytes32>,
}

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
/// # use fuel_streams_core::prelude::*;
/// # use fuel_streams_macros::subject::IntoSubject;
/// let subject = TransactionsSubject {
///     block_height: Some(23.into()),
///     index: Some(1),
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
/// # use fuel_streams_core::prelude::*;
/// let wildcard = TransactionsSubject::wildcard(None, None, Some(Bytes32::zeroed()), None, None);
/// assert_eq!(wildcard, "transactions.*.*.0x0000000000000000000000000000000000000000000000000000000000000000.*.*");
/// ```
///
/// Using the builder pattern:
///
/// ```
/// # use fuel_streams_core::transactions::TransactionsSubject;
/// # use fuel_streams_core::prelude::*;
/// # use fuel_streams_macros::subject::*;
/// let subject = TransactionsSubject::new()
///     .with_block_height(Some(23.into()))
///     .with_index(Some(1))
///     .with_tx_id(Some(Bytes32::zeroed()))
///     .with_status(Some(TransactionStatus::Success))
///     .with_kind(Some(TransactionKind::Script));
/// assert_eq!(subject.parse(), "transactions.23.1.0x0000000000000000000000000000000000000000000000000000000000000000.success.script");
/// ```
#[derive(Subject, Debug, Clone, Default)]
#[subject_wildcard = "transactions.>"]
#[subject_format = "transactions.{block_height}.{index}.{tx_id}.{status}.{kind}"]
pub struct TransactionsSubject {
    pub block_height: Option<BlockHeight>,
    pub index: Option<usize>,
    pub tx_id: Option<Bytes32>,
    pub status: Option<TransactionStatus>,
    pub kind: Option<TransactionKind>,
}

impl From<&Transaction> for TransactionsSubject {
    fn from(transaction: &Transaction) -> Self {
        let subject = TransactionsSubject::new();
        subject
            .with_tx_id(Some(transaction.id.clone()))
            .with_kind(Some(transaction.kind.clone()))
    }
}
