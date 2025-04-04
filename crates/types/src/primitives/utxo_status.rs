use std::str::FromStr;

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
)]
#[serde(rename_all = "snake_case")]
pub enum UtxoStatus {
    #[display("unspent")]
    Unspent,
    #[display("spent")]
    Spent,
}

impl TryFrom<&str> for UtxoStatus {
    type Error = String;
    fn try_from(s: &str) -> Result<Self, Self::Error> {
        match voca_rs::case::snake_case(s).as_str() {
            "unspent" => Ok(UtxoStatus::Unspent),
            "spent" => Ok(UtxoStatus::Spent),
            _ => Err(format!("Unknown UtxoStatus: {}", s)),
        }
    }
}

impl_enum_string_serialization!(UtxoStatus, "utxo_status");

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    use serde_json::json;

    use super::*;

    #[test]
    fn test_deserialize_string_lowercase() {
        let json = json!("unspent");
        let value: UtxoStatus = serde_json::from_value(json).unwrap();
        assert_eq!(value, UtxoStatus::Unspent);
    }

    #[test]
    fn test_deserialize_mixed_case() {
        let json = json!("Unspent");
        let result: Result<UtxoStatus, _> = serde_json::from_value(json);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), UtxoStatus::Unspent);

        let json = json!("UNSPENT");
        let result: Result<UtxoStatus, _> = serde_json::from_value(json);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), UtxoStatus::Unspent);
    }

    #[test]
    fn test_deserialize_invalid() {
        let json = json!("invalid");
        let result: Result<UtxoStatus, _> = serde_json::from_value(json);
        assert!(result.is_err());
    }

    #[test]
    fn test_serialize() {
        let value = UtxoStatus::Unspent;
        let serialized = serde_json::to_value(value).unwrap();
        assert_eq!(serialized, json!("unspent"));
    }

    #[test]
    fn test_case_insensitive_from_str() {
        assert_eq!(
            UtxoStatus::from_str("UNSPENT").unwrap(),
            UtxoStatus::Unspent
        );
        assert_eq!(
            UtxoStatus::from_str("unspent").unwrap(),
            UtxoStatus::Unspent
        );
        assert_eq!(
            UtxoStatus::from_str("Unspent").unwrap(),
            UtxoStatus::Unspent
        );
    }

    #[test]
    fn test_display() {
        assert_eq!(UtxoStatus::Unspent.to_string(), "unspent");
        assert_eq!(UtxoStatus::Spent.to_string(), "spent");
    }
}
