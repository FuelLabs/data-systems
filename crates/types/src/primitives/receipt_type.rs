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
pub enum ReceiptType {
    #[display("call")]
    Call,
    #[display("return")]
    Return,
    #[display("return_data")]
    ReturnData,
    #[display("panic")]
    Panic,
    #[display("revert")]
    Revert,
    #[display("log")]
    Log,
    #[display("log_data")]
    LogData,
    #[display("transfer")]
    Transfer,
    #[display("transfer_out")]
    TransferOut,
    #[display("script_result")]
    ScriptResult,
    #[display("message_out")]
    MessageOut,
    #[display("mint")]
    Mint,
    #[display("burn")]
    Burn,
}

impl TryFrom<&str> for ReceiptType {
    type Error = String;
    fn try_from(s: &str) -> Result<Self, Self::Error> {
        match s.to_lowercase().as_str() {
            "call" => Ok(ReceiptType::Call),
            "return" => Ok(ReceiptType::Return),
            "return_data" => Ok(ReceiptType::ReturnData),
            "panic" => Ok(ReceiptType::Panic),
            "revert" => Ok(ReceiptType::Revert),
            "log" => Ok(ReceiptType::Log),
            "log_data" => Ok(ReceiptType::LogData),
            "transfer" => Ok(ReceiptType::Transfer),
            "transfer_out" => Ok(ReceiptType::TransferOut),
            "script_result" => Ok(ReceiptType::ScriptResult),
            "message_out" => Ok(ReceiptType::MessageOut),
            "mint" => Ok(ReceiptType::Mint),
            "burn" => Ok(ReceiptType::Burn),
            _ => Err(format!("Unknown ReceiptType: {}", s)),
        }
    }
}

impl_enum_string_serialization!(ReceiptType, "receipt_type");

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    use serde_json::json;

    use super::*;

    #[test]
    fn test_deserialize_string_lowercase() {
        let json = json!("call");
        let value: ReceiptType = serde_json::from_value(json).unwrap();
        assert_eq!(value, ReceiptType::Call);
    }

    #[test]
    fn test_deserialize_mixed_case() {
        let json = json!("Call");
        let result: Result<ReceiptType, _> = serde_json::from_value(json);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), ReceiptType::Call);

        let json = json!("CALL");
        let result: Result<ReceiptType, _> = serde_json::from_value(json);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), ReceiptType::Call);
    }

    #[test]
    fn test_deserialize_invalid() {
        let json = json!("invalid");
        let result: Result<ReceiptType, _> = serde_json::from_value(json);
        assert!(result.is_err());
    }

    #[test]
    fn test_serialize() {
        let value = ReceiptType::Call;
        let serialized = serde_json::to_value(value).unwrap();
        assert_eq!(serialized, json!("call"));
    }

    #[test]
    fn test_case_insensitive_from_str() {
        assert_eq!(ReceiptType::from_str("CALL").unwrap(), ReceiptType::Call);
        assert_eq!(ReceiptType::from_str("call").unwrap(), ReceiptType::Call);
        assert_eq!(ReceiptType::from_str("Call").unwrap(), ReceiptType::Call);
    }

    #[test]
    fn test_display() {
        assert_eq!(ReceiptType::Call.to_string(), "call");
        assert_eq!(ReceiptType::Return.to_string(), "return");
        assert_eq!(ReceiptType::ReturnData.to_string(), "return_data");
        assert_eq!(ReceiptType::Panic.to_string(), "panic");
        assert_eq!(ReceiptType::Revert.to_string(), "revert");
        assert_eq!(ReceiptType::Log.to_string(), "log");
        assert_eq!(ReceiptType::LogData.to_string(), "log_data");
        assert_eq!(ReceiptType::Transfer.to_string(), "transfer");
        assert_eq!(ReceiptType::TransferOut.to_string(), "transfer_out");
        assert_eq!(ReceiptType::ScriptResult.to_string(), "script_result");
        assert_eq!(ReceiptType::MessageOut.to_string(), "message_out");
        assert_eq!(ReceiptType::Mint.to_string(), "mint");
        assert_eq!(ReceiptType::Burn.to_string(), "burn");
    }
}
