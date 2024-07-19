use std::collections::HashMap;

use strum::IntoEnumIterator;
use strum_macros::EnumIter;

use crate::types::{BlockHeight, TransactionKind};

pub type SubjectMap = HashMap<SubjectName, Vec<String>>;

#[derive(Debug, EnumIter, Clone, Hash, Eq, PartialEq)]
pub enum SubjectName {
    Blocks,
    Transactions,
}

impl SubjectName {
    pub fn to_vec_string(prefix: &str) -> Vec<String> {
        SubjectName::iter()
            .map(|name| name.with_prefix(prefix))
            .collect()
    }
    pub fn to_map(prefix: &str) -> SubjectMap {
        SubjectName::iter()
            .map(|value| (value.to_owned(), vec![value.with_prefix(prefix)]))
            .collect()
    }

    pub fn with_prefix(&self, prefix: &str) -> String {
        [prefix, &self.to_string()].concat()
    }
    pub fn entity(&self) -> &'static str {
        match self {
            SubjectName::Blocks => "blocks",
            SubjectName::Transactions => "transactions",
        }
    }
}

impl std::fmt::Display for SubjectName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let value: &'static str = match self {
            SubjectName::Blocks => "blocks.*",
            SubjectName::Transactions => "transactions.*.*.*",
        };

        write!(f, "{value}")
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub enum Subject {
    Blocks {
        height: BlockHeight,
    },
    Transactions {
        height: BlockHeight,
        index: usize,
        kind: TransactionKind,
    },
}

impl std::fmt::Display for Subject {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Subject::Blocks { height } => write!(f, "blocks.{height}"),
            Subject::Transactions {
                height,
                index,
                kind,
            } => write!(f, "transactions.{height}.{index}.{kind}"),
        }
    }
}

impl Subject {
    pub fn with_prefix(&self, prefix: &str) -> String {
        [prefix, &self.to_string()].concat()
    }

    #[allow(dead_code)]
    fn get_name(&self) -> SubjectName {
        match self {
            Subject::Blocks { .. } => SubjectName::Blocks,
            Subject::Transactions { .. } => SubjectName::Transactions,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Subjects {
    pub prefix: String,
    pub map: SubjectMap,
}

impl Subjects {
    pub fn new(prefix: &str) -> Self {
        Self {
            prefix: prefix.to_string(),
            map: SubjectName::to_map(prefix),
        }
    }
}
