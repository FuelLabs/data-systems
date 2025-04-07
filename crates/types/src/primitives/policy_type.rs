use std::str::FromStr;

use apache_avro::AvroSchema;
use serde::Serialize;

use crate::impl_enum_string_serialization;

#[derive(
    Debug,
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
pub enum DbPolicyType {
    #[display("tip")]
    Tip,
    #[display("witness_limit")]
    WitnessLimit,
    #[display("maturity")]
    Maturity,
    #[display("max_fee")]
    MaxFee,
}

impl TryFrom<&str> for DbPolicyType {
    type Error = String;
    fn try_from(s: &str) -> Result<Self, Self::Error> {
        match voca_rs::case::snake_case(s).as_str() {
            "tip" => Ok(DbPolicyType::Tip),
            "witness_limit" => Ok(DbPolicyType::WitnessLimit),
            "maturity" => Ok(DbPolicyType::Maturity),
            "max_fee" => Ok(DbPolicyType::MaxFee),
            _ => Err(format!("Unknown DbPolicyType: {}", s)),
        }
    }
}

impl_enum_string_serialization!(DbPolicyType, "policy_type");

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    use serde_json::json;

    use super::*;

    #[test]
    fn test_deserialize_string_lowercase() {
        let json = json!("tip");
        let value: DbPolicyType = serde_json::from_value(json).unwrap();
        assert_eq!(value, DbPolicyType::Tip);
    }

    #[test]
    fn test_deserialize_mixed_case() {
        let json = json!("Tip");
        let result: Result<DbPolicyType, _> = serde_json::from_value(json);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), DbPolicyType::Tip);

        let json = json!("TIP");
        let result: Result<DbPolicyType, _> = serde_json::from_value(json);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), DbPolicyType::Tip);
    }

    #[test]
    fn test_deserialize_invalid() {
        let json = json!("invalid");
        let result: Result<DbPolicyType, _> = serde_json::from_value(json);
        assert!(result.is_err());
    }

    #[test]
    fn test_serialize() {
        let value = DbPolicyType::Tip;
        let serialized = serde_json::to_value(value).unwrap();
        assert_eq!(serialized, json!("tip"));
    }

    #[test]
    fn test_case_insensitive_from_str() {
        assert_eq!(DbPolicyType::from_str("TIP").unwrap(), DbPolicyType::Tip);
        assert_eq!(DbPolicyType::from_str("tip").unwrap(), DbPolicyType::Tip);
        assert_eq!(DbPolicyType::from_str("Tip").unwrap(), DbPolicyType::Tip);
    }

    #[test]
    fn test_display() {
        assert_eq!(DbPolicyType::Tip.to_string(), "tip");
        assert_eq!(DbPolicyType::WitnessLimit.to_string(), "witness_limit");
        assert_eq!(DbPolicyType::Maturity.to_string(), "maturity");
    }
}
