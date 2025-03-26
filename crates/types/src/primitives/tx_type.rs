use serde::{Deserialize, Serialize};

use super::DbTransactionType;
use crate::fuel_core::FuelCoreTransaction;

#[derive(
    Debug, Default, Clone, PartialEq, Serialize, Deserialize, utoipa::ToSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum TransactionType {
    #[default]
    Create,
    Mint,
    Script,
    Upgrade,
    Upload,
    Blob,
}

impl TransactionType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Create => "create",
            Self::Mint => "mint",
            Self::Script => "script",
            Self::Upgrade => "upgrade",
            Self::Upload => "upload",
            Self::Blob => "blob",
        }
    }
}

impl std::fmt::Display for TransactionType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl std::str::FromStr for TransactionType {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            s if s == Self::Create.as_str() => Ok(Self::Create),
            s if s == Self::Mint.as_str() => Ok(Self::Mint),
            s if s == Self::Script.as_str() => Ok(Self::Script),
            s if s == Self::Upgrade.as_str() => Ok(Self::Upgrade),
            s if s == Self::Upload.as_str() => Ok(Self::Upload),
            s if s == Self::Blob.as_str() => Ok(Self::Blob),
            _ => Err(format!("Invalid transaction type: {s}")),
        }
    }
}

impl From<&FuelCoreTransaction> for TransactionType {
    fn from(value: &FuelCoreTransaction) -> Self {
        match value {
            FuelCoreTransaction::Script(_) => TransactionType::Script,
            FuelCoreTransaction::Create(_) => TransactionType::Create,
            FuelCoreTransaction::Mint(_) => TransactionType::Mint,
            FuelCoreTransaction::Upgrade(_) => TransactionType::Upgrade,
            FuelCoreTransaction::Upload(_) => TransactionType::Upload,
            FuelCoreTransaction::Blob(_) => TransactionType::Blob,
        }
    }
}

impl From<&TransactionType> for DbTransactionType {
    fn from(tx: &TransactionType) -> Self {
        match tx {
            TransactionType::Blob => DbTransactionType::Blob,
            TransactionType::Create => DbTransactionType::Create,
            TransactionType::Mint => DbTransactionType::Mint,
            TransactionType::Script => DbTransactionType::Script,
            TransactionType::Upgrade => DbTransactionType::Upgrade,
            TransactionType::Upload => DbTransactionType::Upload,
        }
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    use serde_json::json;

    use super::*;

    #[test]
    fn test_deserialize_string_lowercase() {
        let json = json!("mint");
        let value: TransactionType = serde_json::from_value(json).unwrap();
        assert_eq!(value, TransactionType::Mint);
    }

    #[test]
    fn test_deserialize_invalid() {
        let json = json!("Mint");
        let result: Result<TransactionType, _> = serde_json::from_value(json);
        assert!(result.is_err());

        let json = json!({ "type": "Mint" });
        let result: Result<TransactionType, _> = serde_json::from_value(json);
        assert!(result.is_err());

        let json = json!({ "type": "mint" });
        let result: Result<TransactionType, _> = serde_json::from_value(json);
        assert!(result.is_err());

        let json = json!("invalid");
        let result: Result<TransactionType, _> = serde_json::from_value(json);
        assert!(result.is_err());

        let json = json!({ "type": "invalid" });
        let result: Result<TransactionType, _> = serde_json::from_value(json);
        assert!(result.is_err());

        let json = json!({});
        let result: Result<TransactionType, _> = serde_json::from_value(json);
        assert!(result.is_err());
    }

    #[test]
    fn test_serialize() {
        let value = TransactionType::Mint;
        let serialized = serde_json::to_value(value).unwrap();
        assert_eq!(serialized, json!("mint"));
    }
}
