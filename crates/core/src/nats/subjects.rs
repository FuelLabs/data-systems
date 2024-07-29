use strum::IntoEnumIterator;
use strum_macros::EnumIter;

use crate::types;

#[derive(Debug, EnumIter, Clone, Hash, Eq, PartialEq)]
pub enum SubjectName {
    Blocks,
    Transactions,
    TransactionsById,
}

impl SubjectName {
    pub fn with_prefix(&self, prefix: impl AsRef<str>) -> String {
        [prefix.as_ref(), ".", &self.to_string()].concat()
    }

    pub fn to_vec(prefix: impl AsRef<str>) -> Vec<String> {
        SubjectName::iter()
            .map(|name| name.with_prefix(prefix.as_ref()))
            .collect()
    }
}

impl std::fmt::Display for SubjectName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let value: &'static str = match self {
            // blocks.{producer}.{height}
            SubjectName::Blocks => "blocks.*.*",
            // transactions.{height}.{tx_index}.{tx_id}.{status}.{kind}
            SubjectName::Transactions => "transactions.*.*.*.*.*",
            // by_id.transactions.{id_kind}.{value}
            SubjectName::TransactionsById => "by_id.transactions.*.*",
        };

        write!(f, "{value}")
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub enum Subject {
    Blocks {
        producer: types::Address,
        height: types::BlockHeight,
    },
    Transactions {
        height: types::BlockHeight,
        tx_index: usize,
        tx_id: types::Address,
        status: types::TransactionStatus,
        kind: types::TransactionKind,
    },
    TransactionsById {
        id_kind: types::IdentifierKind,
        value: String,
    },
}

impl Subject {
    pub fn with_prefix(&self, prefix: impl AsRef<str>) -> String {
        [prefix.as_ref(), ".", &self.to_string()].concat()
    }

    #[allow(dead_code)]
    fn get_name(&self) -> SubjectName {
        match self {
            Subject::Blocks { .. } => SubjectName::Blocks,
            Subject::Transactions { .. } => SubjectName::Transactions,
            Subject::TransactionsById { .. } => SubjectName::TransactionsById,
        }
    }
}

impl std::fmt::Display for Subject {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Subject::Blocks { producer, height } => {
                write!(f, "blocks.{producer}.{height}")
            }

            Subject::Transactions {
                height,
                tx_index,
                tx_id,
                status,
                kind,
            } => write!(
                f,
                "transactions.{height}.{tx_index}.{tx_id}.{status}.{kind}"
            ),
            Subject::TransactionsById { id_kind, value } => {
                write!(f, "by_id.transactions.{id_kind}.{value}")
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn subject_name_with_prefix() {
        let prefix = "prefix";
        let subject_name = SubjectName::Blocks;
        assert_eq!(subject_name.with_prefix(prefix), "prefix.blocks.*.*");
    }

    #[test]
    fn subject_name_to_vec() {
        let prefix = "prefix";
        let subject_names = SubjectName::to_vec(prefix);
        assert_eq!(subject_names.len(), 3);
        assert_eq!(subject_names[0], "prefix.blocks.*.*");
        assert_eq!(subject_names[1], "prefix.transactions.*.*.*.*.*");
        assert_eq!(subject_names[2], "prefix.by_id.transactions.*.*");
    }

    #[test]
    fn subject_with_prefix() {
        let prefix = "prefix";
        let subject = Subject::Blocks {
            producer: "producer".to_string(),
            height: 1,
        };
        assert_eq!(subject.with_prefix(prefix), "prefix.blocks.producer.1");
    }

    #[test]
    fn subject_display_blocks() {
        let subject = Subject::Blocks {
            producer: "producer".to_string(),
            height: 1,
        };
        assert_eq!(subject.to_string(), "blocks.producer.1");
    }

    #[test]
    fn subject_display_transactions() {
        let subject = Subject::Transactions {
            height: 1,
            tx_index: 2,
            tx_id: "tx_id".to_string(),
            status: types::TransactionStatus::Success,
            kind: types::TransactionKind::Create,
        };
        assert_eq!(
            subject.to_string(),
            "transactions.1.2.tx_id.success.create"
        );
    }

    #[test]
    fn subject_display_transactions_by_id() {
        let subject = Subject::TransactionsById {
            id_kind: types::IdentifierKind::Address("address".to_string()),
            value: "0x000".to_string(),
        };
        assert_eq!(subject.to_string(), "by_id.transactions.address.0x000");
    }
}
