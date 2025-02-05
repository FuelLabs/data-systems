use serde::{Deserialize, Serialize};

#[derive(thiserror::Error, Debug, PartialEq, Eq)]
pub enum RecordEntityError {
    #[error("Unknown subject: {0}")]
    UnknownSubject(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "record_entity", rename_all = "lowercase")]
pub enum RecordEntity {
    Block,
    Transaction,
    Input,
    Output,
    Receipt,
    Utxo,
}

impl std::fmt::Display for RecordEntity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl RecordEntity {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Block => "block",
            Self::Transaction => "transaction",
            Self::Input => "input",
            Self::Output => "output",
            Self::Receipt => "receipt",
            Self::Utxo => "utxo",
        }
    }

    pub fn table_name(&self) -> String {
        format!("{}{}", self.as_str(), "s")
    }

    pub fn from_subject_id(
        subject: &str,
    ) -> Result<RecordEntity, RecordEntityError> {
        let subject = subject.to_lowercase();
        let subject_entity = if subject.contains("_") {
            subject
                .split("_")
                .next()
                .ok_or(RecordEntityError::UnknownSubject(subject.clone()))?
        } else {
            &subject
        };
        RecordEntity::try_from(subject_entity)
            .map_err(|_| RecordEntityError::UnknownSubject(subject))
    }
}

impl TryFrom<&str> for RecordEntity {
    type Error = RecordEntityError;
    fn try_from(s: &str) -> Result<Self, Self::Error> {
        match s {
            s if s.contains("block") => Ok(Self::Block),
            s if s.contains("transaction") => Ok(Self::Transaction),
            s if s.contains("input") => Ok(Self::Input),
            s if s.contains("output") => Ok(Self::Output),
            s if s.contains("receipt") => Ok(Self::Receipt),
            s if s.contains("utxo") => Ok(Self::Utxo),
            _ => Err(RecordEntityError::UnknownSubject(s.to_string())),
        }
    }
}
