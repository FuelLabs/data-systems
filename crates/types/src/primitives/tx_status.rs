use std::{fmt, str::FromStr};

use serde::{Deserialize, Serialize};

use crate::fuel_core::{
    FuelCoreClientTransactionStatus,
    FuelCoreTransactionStatus,
};

#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TransactionStatus {
    Failed,
    Submitted,
    SqueezedOut,
    Success,
    #[default]
    None,
}

impl TransactionStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            TransactionStatus::Failed => "failed",
            TransactionStatus::Submitted => "submitted",
            TransactionStatus::SqueezedOut => "squeezed_out", /* Corrected to snake_case */
            TransactionStatus::Success => "success",
            TransactionStatus::None => "none",
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
    use pretty_assertions::assert_eq;
    use serde_json::json;

    use super::*;

    #[test]
    fn test_deserialize_string_lowercase() {
        let json = json!("submitted");
        let status: TransactionStatus = serde_json::from_value(json).unwrap();
        assert_eq!(status, TransactionStatus::Submitted);
    }

    #[test]
    fn test_deserialize_invalid() {
        let json = json!("Submitted");
        let result: Result<TransactionStatus, _> = serde_json::from_value(json);
        assert!(result.is_err());

        let json = json!({ "status": "Submitted" });
        let result: Result<TransactionStatus, _> = serde_json::from_value(json);
        assert!(result.is_err());

        let json = json!({ "status": "submitted" });
        let result: Result<TransactionStatus, _> = serde_json::from_value(json);
        assert!(result.is_err());

        let json = json!("squeezedout");
        let result: Result<TransactionStatus, _> = serde_json::from_value(json);
        assert!(result.is_err());

        let json = json!("invalid");
        let result: Result<TransactionStatus, _> = serde_json::from_value(json);
        assert!(result.is_err());

        let json = json!({ "status": "invalid" });
        let result: Result<TransactionStatus, _> = serde_json::from_value(json);
        assert!(result.is_err());

        let json = json!({});
        let result: Result<TransactionStatus, _> = serde_json::from_value(json);
        assert!(result.is_err());
    }

    #[test]
    fn test_serialize() {
        let status = TransactionStatus::SqueezedOut;
        let serialized = serde_json::to_value(status).unwrap();
        assert_eq!(serialized, json!("squeezed_out"));
    }
}
