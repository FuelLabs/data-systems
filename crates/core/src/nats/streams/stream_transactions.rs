use std::fmt;

use super::{stream, subject::Subject};

pub(super) mod subjects {
    use crate::nats::streams::subject;

    #[derive(Debug, Clone, Default)]
    pub struct Transactions {
        height: Option<crate::types::BlockHeight>,
        tx_index: Option<usize>,
        tx_id: Option<crate::types::Address>,
        status: Option<crate::types::TransactionStatus>,
        kind: Option<crate::types::TransactionKind>,
    }
    impl subject::Subject for Transactions {
        const WILDCARD: &'static str = "transaction.*.*.*.*.*";

        fn parse(&self) -> impl ToString {
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
        id_kind: Option<crate::types::IdentifierKind>,
        id_value: Option<String>,
    }
    impl subject::Subject for ById {
        const WILDCARD: &'static str = "by_id.transactions.*.*";

        fn parse(&self) -> impl ToString {
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

impl stream::StreamSubjectsEnum for TransactionSubjects {}
impl stream::StreamIdentifier for stream::Stream<TransactionSubjects> {
    const STREAM: &'static str = "transactions";
}
