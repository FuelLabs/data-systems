use std::str::FromStr;

use apache_avro::AvroSchema;
use serde::Serialize;

use crate::{
    fuel_core::FuelCoreTypesTransaction,
    impl_enum_string_serialization,
};

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
    AvroSchema,
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

impl TryFrom<&str> for TransactionType {
    type Error = String;
    fn try_from(s: &str) -> Result<Self, Self::Error> {
        match voca_rs::case::snake_case(s).as_str() {
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

impl_enum_string_serialization!(TransactionType, "transaction_type");

impl From<&FuelCoreTypesTransaction> for TransactionType {
    fn from(value: &FuelCoreTypesTransaction) -> Self {
        match value {
            FuelCoreTypesTransaction::Script(_) => TransactionType::Script,
            FuelCoreTypesTransaction::Create(_) => TransactionType::Create,
            FuelCoreTypesTransaction::Mint(_) => TransactionType::Mint,
            FuelCoreTypesTransaction::Upgrade(_) => TransactionType::Upgrade,
            FuelCoreTypesTransaction::Upload(_) => TransactionType::Upload,
            FuelCoreTypesTransaction::Blob(_) => TransactionType::Blob,
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
