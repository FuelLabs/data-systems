use std::str::FromStr;

use serde::{Deserialize, Deserializer, Serialize};

use crate::fuel_core::FuelCoreTransaction;

#[derive(
    Debug,
    Default,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Serialize,
    derive_more::Display,
    derive_more::IsVariant,
    utoipa::ToSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum TransactionType {
    #[default]
    #[display("create")]
    Create,
    #[display("mint")]
    Mint,
    #[display("script")]
    Script,
    #[display("upgrade")]
    Upgrade,
    #[display("upload")]
    Upload,
    #[display("blob")]
    Blob,
}

impl<'de> Deserialize<'de> for TransactionType {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        TransactionType::try_from(s.as_str()).map_err(serde::de::Error::custom)
    }
}

impl<'r> sqlx::Decode<'r, sqlx::Postgres> for TransactionType {
    fn decode(
        value: sqlx::postgres::PgValueRef<'r>,
    ) -> Result<Self, Box<dyn std::error::Error + 'static + Send + Sync>> {
        let value = <String as sqlx::Decode<sqlx::Postgres>>::decode(value)?;
        TransactionType::try_from(value.as_str()).map_err(|e| e.into())
    }
}

impl sqlx::Type<sqlx::Postgres> for TransactionType {
    fn type_info() -> sqlx::postgres::PgTypeInfo {
        sqlx::postgres::PgTypeInfo::with_name("transaction_type")
    }
}

impl sqlx::Encode<'_, sqlx::Postgres> for TransactionType {
    fn encode_by_ref(
        &self,
        buf: &mut sqlx::postgres::PgArgumentBuffer,
    ) -> Result<
        sqlx::encode::IsNull,
        Box<dyn std::error::Error + Send + Sync + 'static>,
    > {
        <&str as sqlx::Encode<sqlx::Postgres>>::encode_by_ref(
            &self.to_string().as_str(),
            buf,
        )
    }
}

impl TryFrom<&str> for TransactionType {
    type Error = String;
    fn try_from(s: &str) -> Result<Self, Self::Error> {
        match s.to_lowercase().as_str() {
            "create" => Ok(TransactionType::Create),
            "mint" => Ok(TransactionType::Mint),
            "script" => Ok(TransactionType::Script),
            "upgrade" => Ok(TransactionType::Upgrade),
            "upload" => Ok(TransactionType::Upload),
            "blob" => Ok(TransactionType::Blob),
            _ => Err(format!("Unknown TransactionType: {}", s)),
        }
    }
}

impl FromStr for TransactionType {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        TransactionType::try_from(s)
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
    fn test_deserialize_mixed_case() {
        let json = json!("Mint");
        let result: Result<TransactionType, _> = serde_json::from_value(json);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), TransactionType::Mint);

        let json = json!("MINT");
        let result: Result<TransactionType, _> = serde_json::from_value(json);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), TransactionType::Mint);
    }

    #[test]
    fn test_deserialize_invalid() {
        let json = json!("invalid");
        let result: Result<TransactionType, _> = serde_json::from_value(json);
        assert!(result.is_err());
    }

    #[test]
    fn test_serialize() {
        let value = TransactionType::Mint;
        let serialized = serde_json::to_value(value).unwrap();
        assert_eq!(serialized, json!("mint"));
    }

    #[test]
    fn test_case_insensitive_from_str() {
        assert_eq!(
            TransactionType::from_str("MINT").unwrap(),
            TransactionType::Mint
        );
        assert_eq!(
            TransactionType::from_str("mint").unwrap(),
            TransactionType::Mint
        );
        assert_eq!(
            TransactionType::from_str("Mint").unwrap(),
            TransactionType::Mint
        );
    }

    #[test]
    fn test_display() {
        assert_eq!(TransactionType::Mint.to_string(), "mint");
        assert_eq!(TransactionType::Create.to_string(), "create");
        assert_eq!(TransactionType::Script.to_string(), "script");
    }
}
