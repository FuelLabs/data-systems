use fuel_streams_macros::subject::*;
use fuel_streams_types::*;

/// Represents a subject for logs related to transactions in the Fuel network.
///
/// This subject format allows for efficient querying and filtering of logs
/// based on the block height, transaction ID, the index of the receipt within the transaction,
/// and the unique log ID.
///
/// # Examples
///
/// Creating a subject for a specific log:
///
/// ```
/// # use fuel_streams_core::subjects::*;
/// # use fuel_streams_core::types::*;
/// # use fuel_streams_macros::subject::*;
/// let subject = LogsSubject {
///     block_height: Some(1000.into()),
///     tx_id: Some(Bytes32::from([1u8; 32])),
///     receipt_index: Some(0),
///     log_id: Some(Bytes32::from([2u8; 32])),
/// };
/// assert_eq!(
///     subject.parse(),
///     "logs.1000.0x0101010101010101010101010101010101010101010101010101010101010101.0.0x0202020202020202020202020202020202020202020202020202020202020202"
/// );
/// ```
///
/// Wildcard for querying all logs:
///
/// ```
/// # use fuel_streams_core::subjects::*;
/// # use fuel_streams_macros::subject::*;
/// assert_eq!(LogsSubject::WILDCARD, "logs.>");
/// ```
///
/// Creating a subject query using the `wildcard` method:
///
/// ```
/// # use fuel_streams_core::subjects::*;
/// # use fuel_streams_core::types::*;
/// # use fuel_streams_macros::subject::*;
/// let wildcard = LogsSubject::wildcard(
///     Some(1000.into()),
///     Some(Bytes32::from([1u8; 32])),
///     None,
///     None
/// );
/// assert_eq!(
///     wildcard,
///     "logs.1000.0x0101010101010101010101010101010101010101010101010101010101010101.*.*"
/// );
/// ```
///
/// Using the builder pattern:
///
/// ```
/// # use fuel_streams_core::subjects::*;
/// # use fuel_streams_core::types::*;
/// # use fuel_streams_macros::subject::*;
/// let subject = LogsSubject::new()
///     .with_block_height(Some(2310.into()))
///     .with_tx_id(Some(Bytes32::from([1u8; 32])))
///     .with_receipt_index(Some(0))
///     .with_log_id(Some(Bytes32::from([2u8; 32])));
/// assert_eq!(
///     subject.parse(),
///     "logs.2310.0x0101010101010101010101010101010101010101010101010101010101010101.0.0x0202020202020202020202020202020202020202020202020202020202020202"
/// );
/// ```
#[derive(Subject, Debug, Clone, Default)]
#[subject_wildcard = "logs.>"]
#[subject_format = "logs.{block_height}.{tx_id}.{receipt_index}.{log_id}"]
pub struct LogsSubject {
    pub block_height: Option<BlockHeight>,
    pub tx_id: Option<Bytes32>,
    pub receipt_index: Option<usize>,
    pub log_id: Option<Bytes32>,
}
