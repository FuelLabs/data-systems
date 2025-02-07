use std::{fmt, str::FromStr};

use serde::{
    de::{self, MapAccess, Visitor},
    Deserialize,
    Deserializer,
    Serialize,
};

use crate::fuel_core::{
    FuelCoreClientTransactionStatus,
    FuelCoreTransactionStatus,
};

#[derive(Debug, Default, Clone, PartialEq, Serialize)]
pub enum TransactionStatus {
    Failed,
    Submitted,
    SqueezedOut,
    Success,
    #[default]
    None,
}

impl TransactionStatus {
    fn as_str(&self) -> &'static str {
        match self {
            TransactionStatus::Failed => "failed",
            TransactionStatus::Submitted => "submitted",
            TransactionStatus::SqueezedOut => "squeezed_out", /* Corrected to snake_case */
            TransactionStatus::Success => "success",
            TransactionStatus::None => "none",
        }
    }

    fn from_str_case_insensitive(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "failed" => Some(Self::Failed),
            "submitted" => Some(Self::Submitted),
            "squeezedout" | "squeezed_out" => Some(Self::SqueezedOut), /* Handles both */
            "success" => Some(Self::Success),
            "none" => Some(Self::None),
            _ => None,
        }
    }
}

impl fmt::Display for TransactionStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl FromStr for TransactionStatus {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            s if s == Self::Failed.as_str() => Ok(Self::Failed),
            s if s == Self::Submitted.as_str() => Ok(Self::Submitted),
            s if s == Self::SqueezedOut.as_str() => Ok(Self::SqueezedOut),
            s if s == Self::Success.as_str() => Ok(Self::Success),
            s if s == Self::None.as_str() => Ok(Self::None),
            _ => Err(format!("Invalid transaction status: {s}")),
        }
    }
}

impl<'de> Deserialize<'de> for TransactionStatus {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct TransactionStatusVisitor;

        impl<'de> Visitor<'de> for TransactionStatusVisitor {
            type Value = TransactionStatus;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str(
                    "a string or a map representing TransactionStatus",
                )
            }

            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                // Use the case-insensitive helper function
                TransactionStatus::from_str_case_insensitive(value).ok_or_else(
                    || {
                        E::custom(format!(
                            "invalid TransactionStatus string: {}",
                            value
                        ))
                    },
                )
            }

            fn visit_map<M>(self, mut map: M) -> Result<Self::Value, M::Error>
            where
                M: MapAccess<'de>,
            {
                let mut status: Option<String> = None;

                while let Some(key) = map.next_key::<String>()? {
                    match key.as_str() {
                        "status" => {
                            // No "type" field here, it's "status" directly
                            if status.is_some() {
                                return Err(de::Error::duplicate_field(
                                    "status",
                                ));
                            }
                            status = Some(map.next_value()?);
                        }
                        _ => {
                            let _: serde_json::Value = map.next_value()?; // Ignore other keys
                        }
                    }
                }

                let status_str =
                    status.ok_or_else(|| de::Error::missing_field("status"))?;
                self.visit_str(&status_str) // Reuse string parsing
            }
        }
        deserializer.deserialize_any(TransactionStatusVisitor)
    }
}

impl From<&FuelCoreTransactionStatus> for TransactionStatus {
    fn from(value: &FuelCoreTransactionStatus) -> Self {
        match value {
            FuelCoreTransactionStatus::Failed { .. } => {
                TransactionStatus::Failed
            }
            FuelCoreTransactionStatus::Submitted { .. } => {
                TransactionStatus::Submitted
            }
            FuelCoreTransactionStatus::SqueezedOut { .. } => {
                TransactionStatus::SqueezedOut
            }
            FuelCoreTransactionStatus::Success { .. } => {
                TransactionStatus::Success
            }
        }
    }
}

impl From<&FuelCoreClientTransactionStatus> for TransactionStatus {
    fn from(value: &FuelCoreClientTransactionStatus) -> Self {
        match value {
            FuelCoreClientTransactionStatus::Failure { .. } => {
                TransactionStatus::Failed
            }
            FuelCoreClientTransactionStatus::Submitted { .. } => {
                TransactionStatus::Submitted
            }
            FuelCoreClientTransactionStatus::SqueezedOut { .. } => {
                TransactionStatus::SqueezedOut
            }
            FuelCoreClientTransactionStatus::Success { .. } => {
                TransactionStatus::Success
            }
        }
    }
}

impl From<FuelCoreClientTransactionStatus> for TransactionStatus {
    fn from(value: FuelCoreClientTransactionStatus) -> Self {
        (&value).into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Deserialize, Debug)]
    struct MyStruct {
        status: TransactionStatus,
    }
    use serde_json::json;

    #[test]
    fn test_deserialize_string_lowercase() {
        let json = json!("submitted");
        let status: TransactionStatus = serde_json::from_value(json).unwrap();
        assert_eq!(status, TransactionStatus::Submitted);
    }

    #[test]
    fn test_deserialize_string_uppercase() {
        let json = json!("Submitted");
        let status: TransactionStatus = serde_json::from_value(json).unwrap();
        assert_eq!(status, TransactionStatus::Submitted);
    }
    #[test]
    fn test_deserialize_string_squeezedout() {
        let json = json!("squeezedout");
        let status: TransactionStatus = serde_json::from_value(json).unwrap();
        assert_eq!(status, TransactionStatus::SqueezedOut);
    }
    #[test]
    fn test_deserialize_string_squeezed_out() {
        let json = json!("squeezed_out");
        let status: TransactionStatus = serde_json::from_value(json).unwrap();
        assert_eq!(status, TransactionStatus::SqueezedOut);
    }
    #[test]
    fn test_deserialize_map() {
        let json = json!({ "status": "Success" });
        let status: TransactionStatus = serde_json::from_value(json).unwrap();
        assert_eq!(status, TransactionStatus::Success);
    }

    #[test]
    fn test_deserialize_invalid() {
        let json = json!("invalid");
        let result: Result<TransactionStatus, _> = serde_json::from_value(json);
        assert!(result.is_err());
    }
    #[test]
    fn test_serialize() {
        let status = TransactionStatus::SqueezedOut;
        let serialized = serde_json::to_value(status).unwrap();
        assert_eq!(serialized, json!("SqueezedOut"));
    }

    #[test]
    fn test_deserialize_whole_json_lowercase() {
        let json = json!({ "status": "submitted" });
        let deserialized: MyStruct = serde_json::from_value(json).unwrap();
        assert_eq!(deserialized.status, TransactionStatus::Submitted);
    }

    #[test]
    fn test_deserialize_whole_json_uppercase() {
        let json = json!({ "status": "Submitted" });
        let deserialized: MyStruct = serde_json::from_value(json).unwrap();
        assert_eq!(deserialized.status, TransactionStatus::Submitted);
    }
    #[test]
    fn test_deserialize_whole_json_squeezed_out() {
        let json = json!({ "status": "squeezed_out" });
        let deserialized: MyStruct = serde_json::from_value(json).unwrap();
        assert_eq!(deserialized.status, TransactionStatus::SqueezedOut);
    }
    #[test]
    fn test_deserialize_whole_json_map() {
        let json = json!({ "status": "Submitted" });
        let deserialized: MyStruct = serde_json::from_value(json).unwrap();
        assert_eq!(deserialized.status, TransactionStatus::Submitted);
    }
}
