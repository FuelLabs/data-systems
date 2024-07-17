use strum::IntoEnumIterator;
use strum_macros::EnumIter;

use crate::types::{BlockHeight, TransactionKind};

#[derive(Debug, EnumIter, Clone)]
pub enum SubjectName {
    Blocks,
    Transactions,
}

impl SubjectName {
    pub fn with_prefix(&self, prefix: &str) -> String {
        [prefix, &self.to_string()].concat()
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
    prefix: String,
    subjects: Vec<SubjectName>,
}

impl Subjects {
    pub fn build(prefix: &str, subjects: Vec<SubjectName>) -> Self {
        Self {
            prefix: prefix.to_string(),
            subjects,
        }
    }

    pub fn build_all(prefix: &str) -> Self {
        Self {
            prefix: prefix.to_string(),
            subjects: SubjectName::iter().collect(),
        }
    }

    pub fn to_vec(&self) -> Vec<String> {
        self.subjects
            .iter()
            .map(|name| name.with_prefix(&self.prefix))
            .collect()
    }
}

impl From<Subjects> for Vec<String> {
    fn from(val: Subjects) -> Self {
        val.to_vec()
    }
}
