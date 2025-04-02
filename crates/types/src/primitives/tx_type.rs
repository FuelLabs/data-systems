use serde::{Deserialize, Serialize};

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

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Serialize,
    Deserialize,
    derive_more::Display,
    derive_more::FromStr,
)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum DbTransactionType {
    #[display("SCRIPT")]
    Script,
    #[display("CREATE")]
    Create,
    #[display("MINT")]
    Mint,
    #[display("UPGRADE")]
    Upgrade,
    #[display("UPLOAD")]
    Upload,
    #[display("BLOB")]
    Blob,
}

impl From<TransactionType> for DbTransactionType {
    fn from(value: TransactionType) -> Self {
        match value {
            TransactionType::Script => DbTransactionType::Script,
            TransactionType::Create => DbTransactionType::Create,
            TransactionType::Mint => DbTransactionType::Mint,
            TransactionType::Upgrade => DbTransactionType::Upgrade,
            TransactionType::Upload => DbTransactionType::Upload,
            TransactionType::Blob => DbTransactionType::Blob,
        }
    }
}

impl<'r> sqlx::Decode<'r, sqlx::Postgres> for DbTransactionType {
    fn decode(
        value: sqlx::postgres::PgValueRef<'r>,
    ) -> Result<Self, Box<dyn std::error::Error + 'static + Send + Sync>> {
        let value = <String as sqlx::Decode<sqlx::Postgres>>::decode(value)?;
        match value.as_str() {
            "SCRIPT" => Ok(DbTransactionType::Script),
            "CREATE" => Ok(DbTransactionType::Create),
            "MINT" => Ok(DbTransactionType::Mint),
            "UPGRADE" => Ok(DbTransactionType::Upgrade),
            "UPLOAD" => Ok(DbTransactionType::Upload),
            "BLOB" => Ok(DbTransactionType::Blob),
            _ => Err(format!("Unknown DbTransactionType: {}", value).into()),
        }
    }
}

impl sqlx::Type<sqlx::Postgres> for DbTransactionType {
    fn type_info() -> sqlx::postgres::PgTypeInfo {
        sqlx::postgres::PgTypeInfo::with_name("transaction_type")
    }
}

impl sqlx::Encode<'_, sqlx::Postgres> for DbTransactionType {
    fn encode_by_ref(
        &self,
        buf: &mut sqlx::postgres::PgArgumentBuffer,
    ) -> Result<
        sqlx::encode::IsNull,
        Box<dyn std::error::Error + Send + Sync + 'static>,
    > {
        let s = match self {
            DbTransactionType::Script => "SCRIPT",
            DbTransactionType::Create => "CREATE",
            DbTransactionType::Mint => "MINT",
            DbTransactionType::Upgrade => "UPGRADE",
            DbTransactionType::Upload => "UPLOAD",
            DbTransactionType::Blob => "BLOB",
        };
        <&str as sqlx::Encode<sqlx::Postgres>>::encode_by_ref(&s, buf)
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
