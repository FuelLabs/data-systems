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
pub enum UtxoType {
    #[display("input_contract")]
    InputContract,
    #[display("input_coin")]
    InputCoin,
    #[display("output_coin")]
    OutputCoin,
    #[display("output_variable")]
    OutputVariable,
    #[display("output_change")]
    OutputChange,
}

impl TryFrom<&str> for UtxoType {
    type Error = String;
    fn try_from(s: &str) -> Result<Self, Self::Error> {
        match voca_rs::case::snake_case(s).as_str() {
            "input_contract" => Ok(UtxoType::InputContract),
            "input_coin" => Ok(UtxoType::InputCoin),
            "output_coin" => Ok(UtxoType::OutputCoin),
            "output_variable" => Ok(UtxoType::OutputVariable),
            "output_change" => Ok(UtxoType::OutputChange),
            _ => Err(format!("Unknown UtxoType: {}", s)),
        }
    }
}

impl_enum_string_serialization!(UtxoType, "utxo_type");

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    use serde_json::json;

    use super::*;

    #[test]
    fn test_deserialize_string_lowercase() {
        let json = json!("input_contract");
        let value: UtxoType = serde_json::from_value(json).unwrap();
        assert_eq!(value, UtxoType::InputContract);
    }

    #[test]
    fn test_deserialize_mixed_case() {
        let json = json!("InputContract");
        let result: Result<UtxoType, _> = serde_json::from_value(json);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), UtxoType::InputContract);

        let json = json!("INPUT_CONTRACT");
        let result: Result<UtxoType, _> = serde_json::from_value(json);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), UtxoType::InputContract);
    }

    #[test]
    fn test_deserialize_invalid() {
        let json = json!("invalid");
        let result: Result<UtxoType, _> = serde_json::from_value(json);
        assert!(result.is_err());
    }

    #[test]
    fn test_serialize() {
        let value = UtxoType::InputContract;
        let serialized = serde_json::to_value(value).unwrap();
        assert_eq!(serialized, json!("input_contract"));
    }

    #[test]
    fn test_case_insensitive_from_str() {
        assert_eq!(
            UtxoType::from_str("input_contract").unwrap(),
            UtxoType::InputContract
        );
        assert_eq!(
            UtxoType::from_str("INPUT_CONTRACT").unwrap(),
            UtxoType::InputContract
        );
        assert_eq!(
            UtxoType::from_str("Input_Contract").unwrap(),
            UtxoType::InputContract
        );
        assert_eq!(
            UtxoType::from_str("input_coin").unwrap(),
            UtxoType::InputCoin
        );
        assert_eq!(
            UtxoType::from_str("output_coin").unwrap(),
            UtxoType::OutputCoin
        );
        assert_eq!(
            UtxoType::from_str("output_variable").unwrap(),
            UtxoType::OutputVariable
        );
        assert_eq!(
            UtxoType::from_str("output_change").unwrap(),
            UtxoType::OutputChange
        );
        assert_eq!(
            UtxoType::from_str("OUTPUT_CHANGE").unwrap(),
            UtxoType::OutputChange
        );
    }

    #[test]
    fn test_display() {
        assert_eq!(UtxoType::InputContract.to_string(), "input_contract");
        assert_eq!(UtxoType::InputCoin.to_string(), "input_coin");
        assert_eq!(UtxoType::OutputCoin.to_string(), "output_coin");
        assert_eq!(UtxoType::OutputVariable.to_string(), "output_variable");
        assert_eq!(UtxoType::OutputChange.to_string(), "output_change");
    }
}
