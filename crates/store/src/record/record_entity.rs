use std::str::FromStr;

use serde::{Deserialize, Serialize};

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
}

impl FromStr for RecordEntity {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "block" | "blocks" => Ok(Self::Block),
            "transaction" | "transactions" => Ok(Self::Transaction),
            "input" | "inputs" => Ok(Self::Input),
            "output" | "outputs" => Ok(Self::Output),
            "receipt" | "receipts" => Ok(Self::Receipt),
            "utxo" | "utxos" => Ok(Self::Utxo),
            _ => Err(format!("Invalid record entity: {}", s)),
        }
    }
}
