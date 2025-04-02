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
pub enum InputType {
    #[display("contract")]
    Contract,
    #[display("coin")]
    Coin,
    #[display("message")]
    Message,
}

impl TryFrom<&str> for InputType {
    type Error = String;
    fn try_from(s: &str) -> Result<Self, Self::Error> {
        match s.to_lowercase().as_str() {
            "contract" => Ok(InputType::Contract),
            "coin" => Ok(InputType::Coin),
            "message" => Ok(InputType::Message),
            _ => Err(format!("Unknown InputType: {}", s)),
        }
    }
}

impl_enum_string_serialization!(InputType, "input_type");

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    use serde_json::json;

    use super::*;

    #[test]
    fn test_deserialize_string_lowercase() {
        let json = json!("contract");
        let value: InputType = serde_json::from_value(json).unwrap();
        assert_eq!(value, InputType::Contract);
    }

    #[test]
    fn test_deserialize_mixed_case() {
        let json = json!("Contract");
        let result: Result<InputType, _> = serde_json::from_value(json);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), InputType::Contract);

        let json = json!("CONTRACT");
        let result: Result<InputType, _> = serde_json::from_value(json);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), InputType::Contract);
    }

    #[test]
    fn test_deserialize_invalid() {
        let json = json!("invalid");
        let result: Result<InputType, _> = serde_json::from_value(json);
        assert!(result.is_err());
    }

    #[test]
    fn test_serialize() {
        let value = InputType::Contract;
        let serialized = serde_json::to_value(value).unwrap();
        assert_eq!(serialized, json!("contract"));
    }

    #[test]
    fn test_case_insensitive_from_str() {
        assert_eq!(
            InputType::from_str("CONTRACT").unwrap(),
            InputType::Contract
        );
        assert_eq!(
            InputType::from_str("contract").unwrap(),
            InputType::Contract
        );
        assert_eq!(
            InputType::from_str("Contract").unwrap(),
            InputType::Contract
        );
    }

    #[test]
    fn test_display() {
        assert_eq!(InputType::Contract.to_string(), "contract");
        assert_eq!(InputType::Coin.to_string(), "coin");
        assert_eq!(InputType::Message.to_string(), "message");
    }
}
