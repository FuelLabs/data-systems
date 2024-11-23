use crate::prelude::*;

/// Represents a subject for querying receipts by their identifier in the Fuel ecosystem.
///
/// This struct is used to create and parse subjects related to receipts identified by
/// various types of IDs, which can be used for subscribing to or publishing events
/// about specific receipts.
///
/// # Examples
///
/// Creating and parsing a subject:
///
/// ```
/// # use fuel_streams_core::receipts::subjects::ReceiptsByIdSubject;
/// # use fuel_streams_core::types::*;
/// # use fuel_streams_macros::subject::*;
/// let subject = ReceiptsByIdSubject {
///     tx_id: Some([1u8; 32].into()),
///     index: Some(0),
///     id_kind: Some(IdentifierKind::ContractID),
///     id_value: Some([2u8; 32].into()),
/// };
/// assert_eq!(
///     subject.parse(),
///     "by_id.receipts.0x0101010101010101010101010101010101010101010101010101010101010101.0.contract_id.0x0202020202020202020202020202020202020202020202020202020202020202"
/// );
/// ```
///
/// All receipts by ID wildcard:
///
/// ```
/// # use fuel_streams_core::receipts::subjects::ReceiptsByIdSubject;
/// # use fuel_streams_macros::subject::*;
/// assert_eq!(ReceiptsByIdSubject::WILDCARD, "by_id.receipts.>");
/// ```
///
/// Creating a subject query using the `wildcard` method:
///
/// ```
/// # use fuel_streams_core::receipts::subjects::ReceiptsByIdSubject;
/// # use fuel_streams_core::types::*;
/// # use fuel_streams_macros::subject::*;
/// let wildcard = ReceiptsByIdSubject::wildcard(Some([1u8; 32].into()), Some(0), Some(IdentifierKind::ContractID), None);
/// assert_eq!(wildcard, "by_id.receipts.0x0101010101010101010101010101010101010101010101010101010101010101.0.contract_id.*");
/// ```
///
/// Using the builder pattern:
///
/// ```
/// # use fuel_streams_core::receipts::subjects::ReceiptsByIdSubject;
/// # use fuel_streams_core::types::*;
/// # use fuel_streams_macros::subject::*;
/// let subject = ReceiptsByIdSubject::new()
///     .with_tx_id(Some([1u8; 32].into()))
///     .with_index(Some(0))
///     .with_id_kind(Some(IdentifierKind::ContractID))
///     .with_id_value(Some([2u8; 32].into()));
/// assert_eq!(subject.parse(), "by_id.receipts.0x0101010101010101010101010101010101010101010101010101010101010101.0.contract_id.0x0202020202020202020202020202020202020202020202020202020202020202");
/// ```
#[derive(Subject, Debug, Clone, Default)]
#[subject_wildcard = "by_id.receipts.>"]
#[subject_format = "by_id.receipts.{tx_id}.{index}.{id_kind}.{id_value}"]
pub struct ReceiptsByIdSubject {
    pub tx_id: Option<Bytes32>,
    pub index: Option<u8>,
    pub id_kind: Option<IdentifierKind>,
    pub id_value: Option<Bytes32>,
}

/// Represents a subject for receipts related to contract calls in the Fuel network.
///
/// This subject format allows for efficient querying and filtering of receipts based
/// on the transaction ID, index, the contract initiating the call (`from`), the receiving contract (`to`),
/// and the asset ID involved in the transaction.
///
/// # Examples
///
/// Creating a subject for a specific call receipt:
///
/// ```
/// # use fuel_streams_core::receipts::subjects::ReceiptsCallSubject;
/// # use fuel_streams_core::types::*;
/// # use fuel_streams_macros::subject::*;
/// let subject = ReceiptsCallSubject {
///     tx_id: Some(Bytes32::from([1u8; 32])),
///     index: Some(0),
///     from: Some(ContractId::from([2u8; 32])),
///     to: Some(ContractId::from([3u8; 32])),
///     asset_id: Some(AssetId::from([4u8; 32])),
/// };
/// assert_eq!(
///     subject.parse(),
///     "receipts.0x0101010101010101010101010101010101010101010101010101010101010101.0.call.0x0202020202020202020202020202020202020202020202020202020202020202.0x0303030303030303030303030303030303030303030303030303030303030303.0x0404040404040404040404040404040404040404040404040404040404040404"
/// );
/// ```
///
/// Wildcard for querying all call receipts:
///
/// ```
/// # use fuel_streams_core::receipts::subjects::ReceiptsCallSubject;
/// # use fuel_streams_macros::subject::*;
/// assert_eq!(ReceiptsCallSubject::WILDCARD, "receipts.>");
/// ```
///
/// Creating a subject query using the `wildcard` method:
///
/// ```
/// # use fuel_streams_core::receipts::subjects::ReceiptsCallSubject;
/// # use fuel_streams_core::types::*;
/// # use fuel_streams_macros::subject::*;
/// let wildcard = ReceiptsCallSubject::wildcard(
///     Some(Bytes32::from([1u8; 32])),
///     Some(0),
///     Some(ContractId::from([2u8; 32])),
///     Some(ContractId::from([3u8; 32])),
///     None
/// );
/// assert_eq!(
///     wildcard,
///     "receipts.0x0101010101010101010101010101010101010101010101010101010101010101.0.call.0x0202020202020202020202020202020202020202020202020202020202020202.0x0303030303030303030303030303030303030303030303030303030303030303.*"
/// );
/// ```
///
/// Using the builder pattern:
///
/// ```
/// # use fuel_streams_core::receipts::subjects::ReceiptsCallSubject;
/// # use fuel_streams_core::types::*;
/// # use fuel_streams_macros::subject::*;
/// let subject = ReceiptsCallSubject::new()
///     .with_tx_id(Some(Bytes32::from([1u8; 32])))
///     .with_index(Some(0))
///     .with_from(Some(ContractId::from([2u8; 32])))
///     .with_to(Some(ContractId::from([3u8; 32])))
///     .with_asset_id(Some(AssetId::from([4u8; 32])));
/// assert_eq!(
///     subject.parse(),
///     "receipts.0x0101010101010101010101010101010101010101010101010101010101010101.0.call.0x0202020202020202020202020202020202020202020202020202020202020202.0x0303030303030303030303030303030303030303030303030303030303030303.0x0404040404040404040404040404040404040404040404040404040404040404"
/// );
/// ```
#[derive(Subject, Debug, Clone, Default)]
#[subject_wildcard = "receipts.>"]
#[subject_format = "receipts.{tx_id}.{index}.call.{from}.{to}.{asset_id}"]
pub struct ReceiptsCallSubject {
    pub tx_id: Option<Bytes32>,
    pub index: Option<usize>,
    pub from: Option<ContractId>,
    pub to: Option<ContractId>,
    pub asset_id: Option<AssetId>,
}

/// Represents a subject for receipts related to contract returns in the Fuel network.
///
/// This subject format allows for efficient querying and filtering of return receipts
/// based on the transaction ID, index, and the contract ID (`id`) associated with the return.
///
/// # Examples
///
/// Creating a subject for a specific return receipt:
///
/// ```
/// # use fuel_streams_core::receipts::subjects::ReceiptsReturnSubject;
/// # use fuel_streams_core::types::*;
/// # use fuel_streams_macros::subject::*;
/// let subject = ReceiptsReturnSubject {
///     tx_id: Some(Bytes32::from([1u8; 32])),
///     index: Some(0),
///     id: Some(ContractId::from([2u8; 32])),
/// };
/// assert_eq!(
///     subject.parse(),
///     "receipts.0x0101010101010101010101010101010101010101010101010101010101010101.0.return.0x0202020202020202020202020202020202020202020202020202020202020202"
/// );
/// ```
///
/// Wildcard for querying all return receipts:
///
/// ```
/// # use fuel_streams_core::receipts::subjects::ReceiptsReturnSubject;
/// # use fuel_streams_macros::subject::*;
/// assert_eq!(ReceiptsReturnSubject::WILDCARD, "receipts.>");
/// ```
///
/// Creating a subject query using the `wildcard` method:
///
/// ```
/// # use fuel_streams_core::receipts::subjects::ReceiptsReturnSubject;
/// # use fuel_streams_core::types::*;
/// # use fuel_streams_macros::subject::*;
/// let wildcard = ReceiptsReturnSubject::wildcard(
///     Some(Bytes32::from([1u8; 32])),
///     None,
///     None
/// );
/// assert_eq!(
///     wildcard,
///     "receipts.0x0101010101010101010101010101010101010101010101010101010101010101.*.return.*"
/// );
/// ```
///
/// Using the builder pattern:
///
/// ```
/// # use fuel_streams_core::receipts::subjects::ReceiptsReturnSubject;
/// # use fuel_streams_core::types::*;
/// # use fuel_streams_macros::subject::*;
/// let subject = ReceiptsReturnSubject::new()
///     .with_tx_id(Some(Bytes32::from([1u8; 32])))
///     .with_index(Some(0))
///     .with_id(Some(ContractId::from([2u8; 32])));
/// assert_eq!(
///     subject.parse(),
///     "receipts.0x0101010101010101010101010101010101010101010101010101010101010101.0.return.0x0202020202020202020202020202020202020202020202020202020202020202"
/// );
/// ```
#[derive(Subject, Debug, Clone, Default)]
#[subject_wildcard = "receipts.>"]
#[subject_format = "receipts.{tx_id}.{index}.return.{id}"]
pub struct ReceiptsReturnSubject {
    pub tx_id: Option<Bytes32>,
    pub index: Option<usize>,
    pub id: Option<ContractId>,
}

//
/// This subject format allows for efficient querying and filtering of return data receipts
/// based on the transaction ID, index, and the contract ID (`id`) associated with the return data.
///
/// # Examples
///
/// Creating a subject for a specific return data receipt:
///
/// ```
/// # use fuel_streams_core::receipts::subjects::ReceiptsReturnDataSubject;
/// # use fuel_streams_core::types::*;
/// # use fuel_streams_macros::subject::*;
/// let subject = ReceiptsReturnDataSubject {
///     tx_id: Some(Bytes32::from([1u8; 32])),
///     index: Some(0),
///     id: Some(ContractId::from([2u8; 32])),
/// };
/// assert_eq!(
///     subject.parse(),
///     "receipts.0x0101010101010101010101010101010101010101010101010101010101010101.0.return_data.0x0202020202020202020202020202020202020202020202020202020202020202"
/// );
/// ```
///
/// Wildcard for querying all return data receipts:
///
/// ```
/// # use fuel_streams_core::receipts::subjects::ReceiptsReturnDataSubject;
/// # use fuel_streams_macros::subject::*;
/// assert_eq!(ReceiptsReturnDataSubject::WILDCARD, "receipts.>");
/// ```
///
/// Creating a subject query using the `wildcard` method:
///
/// ```
/// # use fuel_streams_core::receipts::subjects::ReceiptsReturnDataSubject;
/// # use fuel_streams_core::types::*;
/// # use fuel_streams_macros::subject::*;
/// let wildcard = ReceiptsReturnDataSubject::wildcard(
///     Some(Bytes32::from([1u8; 32])),
///     None,
///     None
/// );
/// assert_eq!(
///     wildcard,
///     "receipts.0x0101010101010101010101010101010101010101010101010101010101010101.*.return_data.*"
/// );
/// ```
///
/// Using the builder pattern:
///
/// ```
/// # use fuel_streams_core::receipts::subjects::ReceiptsReturnDataSubject;
/// # use fuel_streams_core::types::*;
/// # use fuel_streams_macros::subject::*;
/// let subject = ReceiptsReturnDataSubject::new()
///     .with_tx_id(Some(Bytes32::from([1u8; 32])))
///     .with_index(Some(0))
///     .with_id(Some(ContractId::from([2u8; 32])));
/// assert_eq!(
///     subject.parse(),
///     "receipts.0x0101010101010101010101010101010101010101010101010101010101010101.0.return_data.0x0202020202020202020202020202020202020202020202020202020202020202"
/// );
/// ```
#[derive(Subject, Debug, Clone, Default)]
#[subject_wildcard = "receipts.>"]
#[subject_format = "receipts.{tx_id}.{index}.return_data.{id}"]
pub struct ReceiptsReturnDataSubject {
    pub tx_id: Option<Bytes32>,
    pub index: Option<usize>,
    pub id: Option<ContractId>,
}

/// Represents a subject for receipts related to contract panics in the Fuel network.
///
/// This subject format allows for efficient querying and filtering of panic receipts
/// based on the transaction ID, index, and the contract ID (`id`) associated with the panic event.
///
/// # Examples
///
/// Creating a subject for a specific panic receipt:
///
/// ```
/// # use fuel_streams_core::receipts::subjects::ReceiptsPanicSubject;
/// # use fuel_streams_core::types::*;
/// # use fuel_streams_macros::subject::*;
/// let subject = ReceiptsPanicSubject {
///     tx_id: Some(Bytes32::from([1u8; 32])),
///     index: Some(0),
///     id: Some(ContractId::from([2u8; 32])),
/// };
/// assert_eq!(
///     subject.parse(),
///     "receipts.0x0101010101010101010101010101010101010101010101010101010101010101.0.panic.0x0202020202020202020202020202020202020202020202020202020202020202"
/// );
/// ```
///
/// Wildcard for querying all panic receipts:
///
/// ```
/// # use fuel_streams_core::receipts::subjects::ReceiptsPanicSubject;
/// # use fuel_streams_macros::subject::*;
/// assert_eq!(ReceiptsPanicSubject::WILDCARD, "receipts.>");
/// ```
///
/// Creating a subject query using the `wildcard` method:
///
/// ```
/// # use fuel_streams_core::receipts::subjects::ReceiptsPanicSubject;
/// # use fuel_streams_core::types::*;
/// # use fuel_streams_macros::subject::*;
/// let wildcard = ReceiptsPanicSubject::wildcard(
///     Some(Bytes32::from([1u8; 32])),
///     None,
///     None
/// );
/// assert_eq!(
///     wildcard,
///     "receipts.0x0101010101010101010101010101010101010101010101010101010101010101.*.panic.*"
/// );
/// ```
///
/// Using the builder pattern:
///
/// ```
/// # use fuel_streams_core::receipts::subjects::ReceiptsPanicSubject;
/// # use fuel_streams_core::types::*;
/// # use fuel_streams_macros::subject::*;
/// let subject = ReceiptsPanicSubject::new()
///     .with_tx_id(Some(Bytes32::from([1u8; 32])))
///     .with_index(Some(0))
///     .with_id(Some(ContractId::from([2u8; 32])));
/// assert_eq!(
///     subject.parse(),
///     "receipts.0x0101010101010101010101010101010101010101010101010101010101010101.0.panic.0x0202020202020202020202020202020202020202020202020202020202020202"
/// );
/// ```
#[derive(Subject, Debug, Clone, Default)]
#[subject_wildcard = "receipts.>"]
#[subject_format = "receipts.{tx_id}.{index}.panic.{id}"]
pub struct ReceiptsPanicSubject {
    pub tx_id: Option<Bytes32>,
    pub index: Option<usize>,
    pub id: Option<ContractId>,
}

/// Represents a subject for receipts related to contract reverts in the Fuel network.
///
/// This subject format allows for efficient querying and filtering of revert receipts
/// based on the transaction ID, index, and the contract ID (`id`) associated with the revert event.
///
/// # Examples
///
/// Creating a subject for a specific revert receipt:
///
/// ```
/// # use fuel_streams_core::receipts::subjects::ReceiptsRevertSubject;
/// # use fuel_streams_core::types::*;
/// # use fuel_streams_macros::subject::*;
/// let subject = ReceiptsRevertSubject {
///     tx_id: Some(Bytes32::from([1u8; 32])),
///     index: Some(0),
///     id: Some(ContractId::from([2u8; 32])),
/// };
/// assert_eq!(
///     subject.parse(),
///     "receipts.0x0101010101010101010101010101010101010101010101010101010101010101.0.revert.0x0202020202020202020202020202020202020202020202020202020202020202"
/// );
/// ```
///
/// Wildcard for querying all revert receipts:
///
/// ```
/// # use fuel_streams_core::receipts::subjects::ReceiptsRevertSubject;
/// # use fuel_streams_macros::subject::*;
/// assert_eq!(ReceiptsRevertSubject::WILDCARD, "receipts.>");
/// ```
///
/// Creating a subject query using the `wildcard` method:
///
/// ```
/// # use fuel_streams_core::receipts::subjects::ReceiptsRevertSubject;
/// # use fuel_streams_core::types::*;
/// # use fuel_streams_macros::subject::*;
/// let wildcard = ReceiptsRevertSubject::wildcard(
///     Some(Bytes32::from([1u8; 32])),
///     None,
///     None
/// );
/// assert_eq!(
///     wildcard,
///     "receipts.0x0101010101010101010101010101010101010101010101010101010101010101.*.revert.*"
/// );
/// ```
///
/// Using the builder pattern:
///
/// ```
/// # use fuel_streams_core::receipts::subjects::ReceiptsRevertSubject;
/// # use fuel_streams_core::types::*;
/// # use fuel_streams_macros::subject::*;
/// let subject = ReceiptsRevertSubject::new()
///     .with_tx_id(Some(Bytes32::from([1u8; 32])))
///     .with_index(Some(0))
///     .with_id(Some(ContractId::from([2u8; 32])));
/// assert_eq!(
///     subject.parse(),
///     "receipts.0x0101010101010101010101010101010101010101010101010101010101010101.0.revert.0x0202020202020202020202020202020202020202020202020202020202020202"
/// );
/// ```
#[derive(Subject, Debug, Clone, Default)]
#[subject_wildcard = "receipts.>"]
#[subject_format = "receipts.{tx_id}.{index}.revert.{id}"]
pub struct ReceiptsRevertSubject {
    pub tx_id: Option<Bytes32>,
    pub index: Option<usize>,
    pub id: Option<ContractId>,
}

/// Represents a subject for log receipts in the Fuel network.
///
/// This subject format allows for efficient querying and filtering of log receipts
/// based on the transaction ID, index, and the contract ID (`id`) associated with the log event.
///
/// # Examples
///
/// Creating a subject for a specific log receipt:
///
/// ```
/// # use fuel_streams_core::receipts::subjects::ReceiptsLogSubject;
/// # use fuel_streams_core::types::*;
/// # use fuel_streams_macros::subject::*;
/// let subject = ReceiptsLogSubject {
///     tx_id: Some(Bytes32::from([1u8; 32])),
///     index: Some(0),
///     id: Some(ContractId::from([2u8; 32])),
/// };
/// assert_eq!(
///     subject.parse(),
///     "receipts.0x0101010101010101010101010101010101010101010101010101010101010101.0.log.0x0202020202020202020202020202020202020202020202020202020202020202"
/// );
/// ```
///
/// Wildcard for querying all log receipts:
///
/// ```
/// # use fuel_streams_core::receipts::subjects::ReceiptsLogSubject;
/// # use fuel_streams_macros::subject::*;
/// assert_eq!(ReceiptsLogSubject::WILDCARD, "receipts.>");
/// ```
///
/// Creating a subject query using the `wildcard` method:
///
/// ```
/// # use fuel_streams_core::receipts::subjects::ReceiptsLogSubject;
/// # use fuel_streams_core::types::*;
/// # use fuel_streams_macros::subject::*;
/// let wildcard = ReceiptsLogSubject::wildcard(
///     Some(Bytes32::from([1u8; 32])),
///     None,
///     None
/// );
/// assert_eq!(
///     wildcard,
///     "receipts.0x0101010101010101010101010101010101010101010101010101010101010101.*.log.*"
/// );
/// ```
///
/// Using the builder pattern:
///
/// ```
/// # use fuel_streams_core::receipts::subjects::ReceiptsLogSubject;
/// # use fuel_streams_core::types::*;
/// # use fuel_streams_macros::subject::*;
/// let subject = ReceiptsLogSubject::new()
///     .with_tx_id(Some(Bytes32::from([1u8; 32])))
///     .with_index(Some(0))
///     .with_id(Some(ContractId::from([2u8; 32])));
/// assert_eq!(
///     subject.parse(),
///     "receipts.0x0101010101010101010101010101010101010101010101010101010101010101.0.log.0x0202020202020202020202020202020202020202020202020202020202020202"
/// );
/// ```
#[derive(Subject, Debug, Clone, Default)]
#[subject_wildcard = "receipts.>"]
#[subject_format = "receipts.{tx_id}.{index}.log.{id}"]
pub struct ReceiptsLogSubject {
    pub tx_id: Option<Bytes32>,
    pub index: Option<usize>,
    pub id: Option<ContractId>,
}

/// Represents a subject for log data receipts in the Fuel network.
///
/// This subject format allows for efficient querying and filtering of log data receipts
/// based on the transaction ID, index, and the contract ID (`id`) associated with the log data.
///
/// # Examples
///
/// Creating a subject for a specific log data receipt:
///
/// ```
/// # use fuel_streams_core::receipts::subjects::ReceiptsLogDataSubject;
/// # use fuel_streams_core::types::*;
/// # use fuel_streams_macros::subject::*;
/// let subject = ReceiptsLogDataSubject {
///     tx_id: Some(Bytes32::from([1u8; 32])),
///     index: Some(0),
///     id: Some(ContractId::from([2u8; 32])),
/// };
/// assert_eq!(
///     subject.parse(),
///     "receipts.0x0101010101010101010101010101010101010101010101010101010101010101.0.log_data.0x0202020202020202020202020202020202020202020202020202020202020202"
/// );
/// ```
///
/// Wildcard for querying all log data receipts:
///
/// ```
/// # use fuel_streams_core::receipts::subjects::ReceiptsLogDataSubject;
/// # use fuel_streams_macros::subject::*;
/// assert_eq!(ReceiptsLogDataSubject::WILDCARD, "receipts.>");
/// ```
///
/// Creating a subject query using the `wildcard` method:
///
/// ```
/// # use fuel_streams_core::receipts::subjects::ReceiptsLogDataSubject;
/// # use fuel_streams_core::types::*;
/// # use fuel_streams_macros::subject::*;
/// let wildcard = ReceiptsLogDataSubject::wildcard(
///     Some(Bytes32::from([1u8; 32])),
///     None,
///     None
/// );
/// assert_eq!(
///     wildcard,
///     "receipts.0x0101010101010101010101010101010101010101010101010101010101010101.*.log_data.*"
/// );
/// ```
///
/// Using the builder pattern:
///
/// ```
/// # use fuel_streams_core::receipts::subjects::ReceiptsLogDataSubject;
/// # use fuel_streams_core::types::*;
/// # use fuel_streams_macros::subject::*;
/// let subject = ReceiptsLogDataSubject::new()
///     .with_tx_id(Some(Bytes32::from([1u8; 32])))
///     .with_index(Some(0))
///     .with_id(Some(ContractId::from([2u8; 32])));
/// assert_eq!(
///     subject.parse(),
///     "receipts.0x0101010101010101010101010101010101010101010101010101010101010101.0.log_data.0x0202020202020202020202020202020202020202020202020202020202020202"
/// );
/// ```
#[derive(Subject, Debug, Clone, Default)]
#[subject_wildcard = "receipts.>"]
#[subject_format = "receipts.{tx_id}.{index}.log_data.{id}"]
pub struct ReceiptsLogDataSubject {
    pub tx_id: Option<Bytes32>,
    pub index: Option<usize>,
    pub id: Option<ContractId>,
}

/// Represents a subject for transfer receipts in the Fuel network.
///
/// This subject format allows for efficient querying and filtering of transfer receipts
/// based on the transaction ID, index, the contract ID of the sender (`from`), the contract ID of the receiver (`to`),
/// and the asset ID involved in the transfer.
///
/// # Examples
///
/// Creating a subject for a specific transfer receipt:
///
/// ```
/// # use fuel_streams_core::receipts::subjects::ReceiptsTransferSubject;
/// # use fuel_streams_core::types::*;
/// # use fuel_streams_macros::subject::*;
/// let subject = ReceiptsTransferSubject {
///     tx_id: Some(Bytes32::from([1u8; 32])),
///     index: Some(0),
///     from: Some(ContractId::from([2u8; 32])),
///     to: Some(ContractId::from([3u8; 32])),
///     asset_id: Some(AssetId::from([4u8; 32])),
/// };
/// assert_eq!(
///     subject.parse(),
///     "receipts.0x0101010101010101010101010101010101010101010101010101010101010101.0.transfer.0x0202020202020202020202020202020202020202020202020202020202020202.0x0303030303030303030303030303030303030303030303030303030303030303.0x0404040404040404040404040404040404040404040404040404040404040404"
/// );
/// ```
///
/// Wildcard for querying all transfer receipts:
///
/// ```
/// # use fuel_streams_core::receipts::subjects::ReceiptsTransferSubject;
/// # use fuel_streams_macros::subject::*;
/// assert_eq!(ReceiptsTransferSubject::WILDCARD, "receipts.>");
/// ```
///
/// Creating a subject query using the `wildcard` method:
///
/// ```
/// # use fuel_streams_core::receipts::subjects::ReceiptsTransferSubject;
/// # use fuel_streams_core::types::*;
/// # use fuel_streams_macros::subject::*;
/// let wildcard = ReceiptsTransferSubject::wildcard(
///     Some(Bytes32::from([1u8; 32])),
///     None,
///     Some(ContractId::from([2u8; 32])),
///     Some(ContractId::from([3u8; 32])),
///     Some(AssetId::from([4u8; 32]))
/// );
/// assert_eq!(
///     wildcard,
///     "receipts.0x0101010101010101010101010101010101010101010101010101010101010101.*.transfer.0x0202020202020202020202020202020202020202020202020202020202020202.0x0303030303030303030303030303030303030303030303030303030303030303.0x0404040404040404040404040404040404040404040404040404040404040404"
/// );
/// ```
///
/// Using the builder pattern:
///
/// ```
/// # use fuel_streams_core::receipts::subjects::ReceiptsTransferSubject;
/// # use fuel_streams_core::types::*;
/// # use fuel_streams_macros::subject::*;
/// let subject = ReceiptsTransferSubject::new()
///     .with_tx_id(Some(Bytes32::from([1u8; 32])))
///     .with_index(Some(0))
///     .with_from(Some(ContractId::from([2u8; 32])))
///     .with_to(Some(ContractId::from([3u8; 32])))
///     .with_asset_id(Some(AssetId::from([4u8; 32])));
/// assert_eq!(
///     subject.parse(),
///     "receipts.0x0101010101010101010101010101010101010101010101010101010101010101.0.transfer.0x0202020202020202020202020202020202020202020202020202020202020202.0x0303030303030303030303030303030303030303030303030303030303030303.0x0404040404040404040404040404040404040404040404040404040404040404"
/// );
/// ```
#[derive(Subject, Debug, Clone, Default)]
#[subject_wildcard = "receipts.>"]
#[subject_format = "receipts.{tx_id}.{index}.transfer.{from}.{to}.{asset_id}"]
pub struct ReceiptsTransferSubject {
    pub tx_id: Option<Bytes32>,
    pub index: Option<usize>,
    pub from: Option<ContractId>,
    pub to: Option<ContractId>,
    pub asset_id: Option<AssetId>,
}

/// Represents a subject for transfer-out receipts in the Fuel network.
///
/// This subject format allows for efficient querying and filtering of transfer-out receipts
/// based on the transaction ID, index, the contract ID of the sender (`from`), the address of the receiver (`to`),
/// and the asset ID involved in the transfer-out.
///
/// # Examples
///
/// Creating a subject for a specific transfer-out receipt:
///
/// ```
/// # use fuel_streams_core::receipts::subjects::ReceiptsTransferOutSubject;
/// # use fuel_streams_core::types::*;
/// # use fuel_streams_macros::subject::*;
/// let subject = ReceiptsTransferOutSubject {
///     tx_id: Some(Bytes32::from([1u8; 32])),
///     index: Some(0),
///     from: Some(ContractId::from([2u8; 32])),
///     to: Some(Address::from([3u8; 32])),
///     asset_id: Some(AssetId::from([4u8; 32])),
/// };
/// assert_eq!(
///     subject.parse(),
///     "receipts.0x0101010101010101010101010101010101010101010101010101010101010101.0.transfer_out.0x0202020202020202020202020202020202020202020202020202020202020202.0x0303030303030303030303030303030303030303030303030303030303030303.0x0404040404040404040404040404040404040404040404040404040404040404"
/// );
/// ```
///
/// Wildcard for querying all transfer-out receipts:
///
/// ```
/// # use fuel_streams_core::receipts::subjects::ReceiptsTransferOutSubject;
/// # use fuel_streams_macros::subject::*;
/// assert_eq!(ReceiptsTransferOutSubject::WILDCARD, "receipts.>");
/// ```
///
/// Creating a subject query using the `wildcard` method:
///
/// ```
/// # use fuel_streams_core::receipts::subjects::ReceiptsTransferOutSubject;
/// # use fuel_streams_core::types::*;
/// # use fuel_streams_macros::subject::*;
/// let wildcard = ReceiptsTransferOutSubject::wildcard(
///     Some(Bytes32::from([1u8; 32])),
///     None,
///     Some(ContractId::from([2u8; 32])),
///     Some(Address::from([3u8; 32])),
///     Some(AssetId::from([4u8; 32]))
/// );
/// assert_eq!(
///     wildcard,
///     "receipts.0x0101010101010101010101010101010101010101010101010101010101010101.*.transfer_out.0x0202020202020202020202020202020202020202020202020202020202020202.0x0303030303030303030303030303030303030303030303030303030303030303.0x0404040404040404040404040404040404040404040404040404040404040404"
/// );
/// ```
///
/// Using the builder pattern:
///
/// ```
/// # use fuel_streams_core::receipts::subjects::ReceiptsTransferOutSubject;
/// # use fuel_streams_core::types::*;
/// # use fuel_streams_macros::subject::*;
/// let subject = ReceiptsTransferOutSubject::new()
///     .with_tx_id(Some(Bytes32::from([1u8; 32])))
///     .with_index(Some(0))
///     .with_from(Some(ContractId::from([2u8; 32])))
///     .with_to(Some(Address::from([3u8; 32])))
///     .with_asset_id(Some(AssetId::from([4u8; 32])));
/// assert_eq!(
///     subject.parse(),
///     "receipts.0x0101010101010101010101010101010101010101010101010101010101010101.0.transfer_out.0x0202020202020202020202020202020202020202020202020202020202020202.0x0303030303030303030303030303030303030303030303030303030303030303.0x0404040404040404040404040404040404040404040404040404040404040404"
/// );
/// ```
#[derive(Subject, Debug, Clone, Default)]
#[subject_wildcard = "receipts.>"]
#[subject_format = "receipts.{tx_id}.{index}.transfer_out.{from}.{to}.{asset_id}"]
pub struct ReceiptsTransferOutSubject {
    pub tx_id: Option<Bytes32>,
    pub index: Option<usize>,
    pub from: Option<ContractId>,
    pub to: Option<Address>,
    pub asset_id: Option<AssetId>,
}

/// Represents a subject for script result receipts in the Fuel network.
///
/// This subject format allows for efficient querying and filtering of script result receipts
/// based on the transaction ID (`tx_id`) and index (`index`) within the transaction.
///
/// # Examples
///
/// Creating a subject for a specific script result receipt:
///
/// ```
/// # use fuel_streams_core::receipts::subjects::ReceiptsScriptResultSubject;
/// # use fuel_streams_core::types::*;
/// # use fuel_streams_macros::subject::*;
/// let subject = ReceiptsScriptResultSubject {
///     tx_id: Some(Bytes32::from([1u8; 32])),
///     index: Some(0),
/// };
/// assert_eq!(
///     subject.parse(),
///     "receipts.0x0101010101010101010101010101010101010101010101010101010101010101.0.script_result"
/// );
/// ```
///
/// Wildcard for querying all script result receipts:
///
/// ```
/// # use fuel_streams_core::receipts::subjects::ReceiptsScriptResultSubject;
/// # use fuel_streams_macros::subject::*;
/// assert_eq!(ReceiptsScriptResultSubject::WILDCARD, "receipts.>");
/// ```
///
/// Creating a subject query using the `wildcard` method:
///
/// ```
/// # use fuel_streams_core::receipts::subjects::ReceiptsScriptResultSubject;
/// # use fuel_streams_core::types::*;
/// # use fuel_streams_macros::subject::*;
/// let wildcard = ReceiptsScriptResultSubject::wildcard(
///     Some(Bytes32::from([1u8; 32])),
///     None
/// );
/// assert_eq!(
///     wildcard,
///     "receipts.0x0101010101010101010101010101010101010101010101010101010101010101.*.script_result"
/// );
/// ```
///
/// Using the builder pattern:
///
/// ```
/// # use fuel_streams_core::receipts::subjects::ReceiptsScriptResultSubject;
/// # use fuel_streams_core::types::*;
/// # use fuel_streams_macros::subject::*;
/// let subject = ReceiptsScriptResultSubject::new()
///     .with_tx_id(Some(Bytes32::from([1u8; 32])))
///     .with_index(Some(0));
/// assert_eq!(
///     subject.parse(),
///     "receipts.0x0101010101010101010101010101010101010101010101010101010101010101.0.script_result"
/// );
/// ```
#[derive(Subject, Debug, Clone, Default)]
#[subject_wildcard = "receipts.>"]
#[subject_format = "receipts.{tx_id}.{index}.script_result"]
pub struct ReceiptsScriptResultSubject {
    pub tx_id: Option<Bytes32>,
    pub index: Option<usize>,
}

/// Represents a subject for message-out receipts in the Fuel network.
///
/// This subject format allows for efficient querying and filtering of message-out receipts
/// based on the transaction ID (`tx_id`), index (`index`), sender address (`sender`), and recipient address (`recipient`).
///
/// # Examples
///
/// Creating a subject for a specific message-out receipt:
///
/// ```
/// # use fuel_streams_core::receipts::subjects::ReceiptsMessageOutSubject;
/// # use fuel_streams_core::types::*;
/// # use fuel_streams_macros::subject::*;
/// let subject = ReceiptsMessageOutSubject {
///     tx_id: Some(Bytes32::from([1u8; 32])),
///     index: Some(0),
///     sender: Some(Address::from([2u8; 32])),
///     recipient: Some(Address::from([3u8; 32])),
/// };
/// assert_eq!(
///     subject.parse(),
///     "receipts.0x0101010101010101010101010101010101010101010101010101010101010101.0.message_out.0x0202020202020202020202020202020202020202020202020202020202020202.0x0303030303030303030303030303030303030303030303030303030303030303"
/// );
/// ```
///
/// Wildcard for querying all message-out receipts:
///
/// ```
/// # use fuel_streams_core::receipts::subjects::ReceiptsMessageOutSubject;
/// # use fuel_streams_macros::subject::*;
/// assert_eq!(ReceiptsMessageOutSubject::WILDCARD, "receipts.>");
/// ```
///
/// Creating a subject query using the `wildcard` method:
///
/// ```
/// # use fuel_streams_core::receipts::subjects::ReceiptsMessageOutSubject;
/// # use fuel_streams_core::types::*;
/// # use fuel_streams_macros::subject::*;
/// let wildcard = ReceiptsMessageOutSubject::wildcard(
///     Some(Bytes32::from([1u8; 32])),
///     None,
///     Some(Address::from([2u8; 32])),
///     Some(Address::from([3u8; 32])),
/// );
/// assert_eq!(
///     wildcard,
///     "receipts.0x0101010101010101010101010101010101010101010101010101010101010101.*.message_out.0x0202020202020202020202020202020202020202020202020202020202020202.0x0303030303030303030303030303030303030303030303030303030303030303"
/// );
/// ```
///
/// Using the builder pattern:
///
/// ```
/// # use fuel_streams_core::receipts::subjects::ReceiptsMessageOutSubject;
/// # use fuel_streams_core::types::*;
/// # use fuel_streams_macros::subject::*;
/// let subject = ReceiptsMessageOutSubject::new()
///     .with_tx_id(Some(Bytes32::from([1u8; 32])))
///     .with_index(Some(0))
///     .with_sender(Some(Address::from([2u8; 32])))
///     .with_recipient(Some(Address::from([3u8; 32])));
/// assert_eq!(
///     subject.parse(),
///     "receipts.0x0101010101010101010101010101010101010101010101010101010101010101.0.message_out.0x0202020202020202020202020202020202020202020202020202020202020202.0x0303030303030303030303030303030303030303030303030303030303030303"
/// );
/// ```
#[derive(Subject, Debug, Clone, Default)]
#[subject_wildcard = "receipts.>"]
#[subject_format = "receipts.{tx_id}.{index}.message_out.{sender}.{recipient}"]
pub struct ReceiptsMessageOutSubject {
    pub tx_id: Option<Bytes32>,
    pub index: Option<usize>,
    pub sender: Option<Address>,
    pub recipient: Option<Address>,
}

/// Represents a subject for mint receipts in the Fuel network.
///
/// This subject format allows for efficient querying and filtering of mint receipts
/// based on the transaction ID (`tx_id`), index (`index`), contract ID (`contract_id`), and sub ID (`sub_id`).
///
/// # Examples
///
/// Creating a subject for a specific mint receipt:
///
/// ```
/// # use fuel_streams_core::receipts::subjects::ReceiptsMintSubject;
/// # use fuel_streams_core::types::*;
/// # use fuel_streams_macros::subject::*;
/// let subject = ReceiptsMintSubject {
///     tx_id: Some(Bytes32::from([1u8; 32])),
///     index: Some(0),
///     contract_id: Some(ContractId::from([2u8; 32])),
///     sub_id: Some(Bytes32::from([3u8; 32])),
/// };
/// assert_eq!(
///     subject.parse(),
///     "receipts.0x0101010101010101010101010101010101010101010101010101010101010101.0.mint.0x0202020202020202020202020202020202020202020202020202020202020202.0x0303030303030303030303030303030303030303030303030303030303030303"
/// );
/// ```
///
/// Wildcard for querying all mint receipts:
///
/// ```
/// # use fuel_streams_core::receipts::subjects::ReceiptsMintSubject;
/// # use fuel_streams_macros::subject::*;
/// assert_eq!(ReceiptsMintSubject::WILDCARD, "receipts.>");
/// ```
///
/// Creating a subject query using the `wildcard` method:
///
/// ```
/// # use fuel_streams_core::receipts::subjects::ReceiptsMintSubject;
/// # use fuel_streams_core::types::*;
/// # use fuel_streams_macros::subject::*;
/// let wildcard = ReceiptsMintSubject::wildcard(
///     Some(Bytes32::from([1u8; 32])),
///     None,
///     Some(ContractId::from([2u8; 32])),
///     Some(Bytes32::from([3u8; 32])),
/// );
/// assert_eq!(
///     wildcard,
///     "receipts.0x0101010101010101010101010101010101010101010101010101010101010101.*.mint.0x0202020202020202020202020202020202020202020202020202020202020202.0x0303030303030303030303030303030303030303030303030303030303030303"
/// );
/// ```
///
/// Using the builder pattern:
///
/// ```
/// # use fuel_streams_core::receipts::subjects::ReceiptsMintSubject;
/// # use fuel_streams_core::types::*;
/// # use fuel_streams_macros::subject::*;
/// let subject = ReceiptsMintSubject::new()
///     .with_tx_id(Some(Bytes32::from([1u8; 32])))
///     .with_index(Some(0))
///     .with_contract_id(Some(ContractId::from([2u8; 32])))
///     .with_sub_id(Some(Bytes32::from([3u8; 32])));
/// assert_eq!(
///     subject.parse(),
///     "receipts.0x0101010101010101010101010101010101010101010101010101010101010101.0.mint.0x0202020202020202020202020202020202020202020202020202020202020202.0x0303030303030303030303030303030303030303030303030303030303030303"
/// );
/// ```
#[derive(Subject, Debug, Clone, Default)]
#[subject_wildcard = "receipts.>"]
#[subject_format = "receipts.{tx_id}.{index}.mint.{contract_id}.{sub_id}"]
pub struct ReceiptsMintSubject {
    pub tx_id: Option<Bytes32>,
    pub index: Option<usize>,
    pub contract_id: Option<ContractId>,
    pub sub_id: Option<Bytes32>,
}

/// Represents a subject for burn receipts in the Fuel network.
///
/// This subject format allows for efficient querying and filtering of burn receipts
/// based on the transaction ID (`tx_id`), index (`index`), contract ID (`contract_id`), and sub ID (`sub_id`).
///
/// # Examples
///
/// Creating a subject for a specific burn receipt:
///
/// ```
/// # use fuel_streams_core::receipts::subjects::ReceiptsBurnSubject;
/// # use fuel_streams_core::types::*;
/// # use fuel_streams_macros::subject::*;
/// let subject = ReceiptsBurnSubject {
///     tx_id: Some(Bytes32::from([1u8; 32])),
///     index: Some(0),
///     contract_id: Some(ContractId::from([2u8; 32])),
///     sub_id: Some(Bytes32::from([3u8; 32])),
/// };
/// assert_eq!(
///     subject.parse(),
///     "receipts.0x0101010101010101010101010101010101010101010101010101010101010101.0.burn.0x0202020202020202020202020202020202020202020202020202020202020202.0x0303030303030303030303030303030303030303030303030303030303030303"
/// );
/// ```
///
/// Wildcard for querying all burn receipts:
///
/// ```
/// # use fuel_streams_core::receipts::subjects::ReceiptsBurnSubject;
/// # use fuel_streams_macros::subject::*;
/// assert_eq!(ReceiptsBurnSubject::WILDCARD, "receipts.>");
/// ```
///
/// Creating a subject query using the `wildcard` method:
///
/// ```
/// # use fuel_streams_core::receipts::subjects::ReceiptsBurnSubject;
/// # use fuel_streams_core::types::*;
/// # use fuel_streams_macros::subject::*;
/// let wildcard = ReceiptsBurnSubject::wildcard(
///     Some(Bytes32::from([1u8; 32])),
///     None,
///     Some(ContractId::from([2u8; 32])),
///     Some(Bytes32::from([3u8; 32])),
/// );
/// assert_eq!(
///     wildcard,
///     "receipts.0x0101010101010101010101010101010101010101010101010101010101010101.*.burn.0x0202020202020202020202020202020202020202020202020202020202020202.0x0303030303030303030303030303030303030303030303030303030303030303"
/// );
/// ```
///
/// Using the builder pattern:
///
/// ```
/// # use fuel_streams_core::receipts::subjects::ReceiptsBurnSubject;
/// # use fuel_streams_core::types::*;
/// # use fuel_streams_macros::subject::*;
/// let subject = ReceiptsBurnSubject::new()
///     .with_tx_id(Some(Bytes32::from([1u8; 32])))
///     .with_index(Some(0))
///     .with_contract_id(Some(ContractId::from([2u8; 32])))
///     .with_sub_id(Some(Bytes32::from([3u8; 32])));
/// assert_eq!(
///     subject.parse(),
///     "receipts.0x0101010101010101010101010101010101010101010101010101010101010101.0.burn.0x0202020202020202020202020202020202020202020202020202020202020202.0x0303030303030303030303030303030303030303030303030303030303030303"
/// );
/// ```
#[derive(Subject, Debug, Clone, Default)]
#[subject_wildcard = "receipts.>"]
#[subject_format = "receipts.{tx_id}.{index}.burn.{contract_id}.{sub_id}"]
pub struct ReceiptsBurnSubject {
    pub tx_id: Option<Bytes32>,
    pub index: Option<usize>,
    pub contract_id: Option<ContractId>,
    pub sub_id: Option<Bytes32>,
}
