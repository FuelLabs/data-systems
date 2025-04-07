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
pub enum OutputType {
    #[display("coin")]
    Coin,
    #[display("contract")]
    Contract,
    #[display("change")]
    Change,
    #[display("variable")]
    Variable,
    #[display("contract_created")]
    ContractCreated,
}

impl TryFrom<&str> for OutputType {
    type Error = String;
    fn try_from(s: &str) -> Result<Self, Self::Error> {
        match voca_rs::case::snake_case(s).as_str() {
            "coin" => Ok(OutputType::Coin),
            "contract" => Ok(OutputType::Contract),
            "change" => Ok(OutputType::Change),
            "variable" => Ok(OutputType::Variable),
            "contract_created" => Ok(OutputType::ContractCreated),
            _ => Err(format!("Unknown OutputType: {}", s)),
        }
    }
}

impl_enum_string_serialization!(OutputType, "output_type");

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    use serde_json::json;

    use super::*;

    #[test]
    fn test_deserialize_string_lowercase() {
        let json = json!("contract");
        let value: OutputType = serde_json::from_value(json).unwrap();
        assert_eq!(value, OutputType::Contract);
    }

    #[test]
    fn test_deserialize_mixed_case() {
        let json = json!("Contract");
        let result: Result<OutputType, _> = serde_json::from_value(json);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), OutputType::Contract);

        let json = json!("CONTRACT");
        let result: Result<OutputType, _> = serde_json::from_value(json);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), OutputType::Contract);
    }

    #[test]
    fn test_deserialize_invalid() {
        let json = json!("invalid");
        let result: Result<OutputType, _> = serde_json::from_value(json);
        assert!(result.is_err());
    }

    #[test]
    fn test_serialize() {
        let value = OutputType::Contract;
        let serialized = serde_json::to_value(value).unwrap();
        assert_eq!(serialized, json!("contract"));
    }

    #[test]
    fn test_case_insensitive_from_str() {
        assert_eq!(
            OutputType::from_str("CONTRACT").unwrap(),
            OutputType::Contract
        );
        assert_eq!(
            OutputType::from_str("contract").unwrap(),
            OutputType::Contract
        );
        assert_eq!(
            OutputType::from_str("Contract").unwrap(),
            OutputType::Contract
        );
    }

    #[test]
    fn test_display() {
        assert_eq!(OutputType::Coin.to_string(), "coin");
        assert_eq!(OutputType::Contract.to_string(), "contract");
        assert_eq!(OutputType::Change.to_string(), "change");
        assert_eq!(OutputType::Variable.to_string(), "variable");
        assert_eq!(OutputType::ContractCreated.to_string(), "contract_created");
    }
}
