use std::fmt;

use super::{stream, subject::Subject};

pub mod subjects {
    use crate::nats::streams::subject;

    #[derive(Debug, Clone, Default)]
    pub struct Transactions {
        pub height: Option<crate::types::BlockHeight>,
        pub tx_index: Option<usize>,
        pub tx_id: Option<crate::types::Address>,
        pub status: Option<crate::types::TransactionStatus>,
        pub kind: Option<crate::types::TransactionKind>,
    }
    impl subject::Subject for Transactions {
        const WILDCARD: &'static str = "transactions.*.*.*.*.*";

        fn parse(&self) -> String {
            let height = subject::parse_param(&self.height);
            let tx_index = subject::parse_param(&self.tx_index);
            let tx_id = subject::parse_param(&self.tx_id);
            let status = subject::parse_param(&self.status);
            let kind = subject::parse_param(&self.kind);
            format!("transactions.{height}.{tx_index}.{tx_id}.{status}.{kind}")
        }
    }

    #[derive(Debug, Clone, Default)]
    pub struct ById {
        pub id_kind: Option<crate::types::IdentifierKind>,
        pub id_value: Option<String>,
    }
    impl subject::Subject for ById {
        const WILDCARD: &'static str = "by_id.transactions.*.*";

        fn parse(&self) -> String {
            let id_kind = subject::parse_param(&self.id_kind);
            let id_value = subject::parse_param(&self.id_value);
            format!("by_id.transactions.{id_kind}.{id_value}")
        }
    }
}

#[derive(Debug, Clone, strum::EnumIter)]
pub enum TransactionSubjects {
    Transactions(subjects::Transactions),
    TransactionsById(subjects::ById),
}

impl fmt::Display for TransactionSubjects {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let value = match self {
            TransactionSubjects::Transactions(s) => s.wildcard(),
            TransactionSubjects::TransactionsById(s) => s.wildcard(),
        };
        write!(f, "{}", value)
    }
}

impl stream::StreamSubjects for TransactionSubjects {}
impl stream::StreamIdentifier for stream::Stream<TransactionSubjects> {
    const STREAM: &'static str = "transactions";
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;
    use crate::types::{
        BoxedResult,
        IdentifierKind,
        TransactionKind,
        TransactionStatus,
    };

    #[test]
    fn can_parse_subjecs() -> BoxedResult<()> {
        let subject_transactions = subjects::Transactions {
            height: Some(100_u32),
            tx_index: Some(1),
            tx_id: Some("0x000".to_string()),
            status: Some(TransactionStatus::Success),
            kind: Some(TransactionKind::Script),
        };
        let parsed = subject_transactions.parse();
        assert_eq!(parsed, "transactions.100.1.0x000.success.script");

        let subject_by_id = subjects::ById {
            id_kind: Some(IdentifierKind::ContractID),
            id_value: Some("0x000".to_string()),
        };
        let parsed = subject_by_id.parse();
        assert_eq!(parsed, "by_id.transactions.contract_id.0x000");

        Ok(())
    }
}
