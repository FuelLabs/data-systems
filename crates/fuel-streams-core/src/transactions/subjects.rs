use fuel_core_types::fuel_tx::UniqueIdentifier;
use fuel_streams_macros::subject::{IntoSubject, Subject};

use crate::{blocks::types::BlockHeight, types::*};

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
    fn transactions_subjects_all() {
        assert_eq!(TransactionsSubject::WILDCARD, "transactions.>");
        assert_eq!(TransactionsByIdSubject::WILDCARD, "by_id.transactions.>");
    }

    #[test]
    fn transactions_subjects_parse() {
        let transactions_subject = TransactionsSubject {
            height: Some(23.into()),
            tx_index: Some(1),
            tx_id: Some(Bytes32::zeroed()),
            status: Some(TransactionStatus::Success),
            kind: Some(TransactionKind::Script),
        };
        assert_eq!(
            transactions_subject.parse(),
            "transactions.23.1.0x0000000000000000000000000000000000000000000000000000000000000000.success.script"
        );

        let transactions_by_id_subject = TransactionsByIdSubject {
            id_kind: Some(IdentifierKind::ContractID),
            id_value: Some(Address::zeroed()),
        };
        assert_eq!(
            transactions_by_id_subject.parse(),
            "by_id.transactions.contract_id.0x0000000000000000000000000000000000000000000000000000000000000000"
        )
    }

    #[test]
    fn transactions_subjects_wildcard() {
        let wildcard1 = TransactionsSubject::wildcard(
            None,
            None,
            Some(Bytes32::zeroed()),
            None,
            None,
        );
        assert_eq!(wildcard1, "transactions.*.*.0x0000000000000000000000000000000000000000000000000000000000000000.*.*");

        let wildcard2 = TransactionsByIdSubject::wildcard(
            Some(IdentifierKind::ContractID),
            None,
        );
        assert_eq!(wildcard2, "by_id.transactions.contract_id.*");
    }

    #[test]
    fn transactions_subjects_builder() {
        let transactions_subject =
            TransactionsSubject::new().with_height(Some(23.into()));
        assert_eq!(transactions_subject.parse(), "transactions.23.*.*.*.*");

        let transactions_by_id_subject = TransactionsByIdSubject::new()
            .with_id_kind(Some(IdentifierKind::ContractID));
        assert_eq!(
            transactions_by_id_subject.parse(),
            "by_id.transactions.contract_id.*"
        );
    }

    #[test]
    fn transactions_subject_from_transaction() {
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
