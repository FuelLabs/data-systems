use std::str::FromStr;

use serde::Serialize;

use crate::{
    fuel_core::{
        FuelCoreClientTransactionStatus,
        FuelCoreTransactionExecutionStatus,
    },
    impl_enum_string_serialization,
};

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
pub enum TransactionStatus {
    #[display("pre_confirmation_failed")]
    PreConfirmationFailed,
    #[display("pre_confirmation_success")]
    PreConfirmationSuccess,
    #[display("failed")]
    Failed,
    #[display("submitted")]
    Submitted,
    #[display("squeezed_out")]
    SqueezedOut,
    #[display("success")]
    Success,
}

impl TryFrom<&str> for TransactionStatus {
    type Error = String;
    fn try_from(s: &str) -> Result<Self, Self::Error> {
        match s.to_lowercase().as_str() {
            "pre_confirmation_failed" => {
                Ok(TransactionStatus::PreConfirmationFailed)
            }
            "pre_confirmation_success" => {
                Ok(TransactionStatus::PreConfirmationSuccess)
            }
            "failed" => Ok(TransactionStatus::Failed),
            "submitted" => Ok(TransactionStatus::Submitted),
            "squeezed_out" => Ok(TransactionStatus::SqueezedOut),
            "success" => Ok(TransactionStatus::Success),
            _ => Err(format!("Unknown TransactionStatus: {}", s)),
        }
    }
}

impl_enum_string_serialization!(TransactionStatus, "transaction_status");

impl From<&FuelCoreTransactionExecutionStatus> for TransactionStatus {
    fn from(value: &FuelCoreTransactionExecutionStatus) -> Self {
        match value {
            FuelCoreTransactionExecutionStatus::Failed { .. } => {
                TransactionStatus::Failed
            }
            FuelCoreTransactionExecutionStatus::Submitted { .. } => {
                TransactionStatus::Submitted
            }
            FuelCoreTransactionExecutionStatus::SqueezedOut { .. } => {
                TransactionStatus::SqueezedOut
            }
            FuelCoreTransactionExecutionStatus::Success { .. } => {
                TransactionStatus::Success
            }
        }
    }
}

impl From<&FuelCoreClientTransactionStatus> for TransactionStatus {
    fn from(value: &FuelCoreClientTransactionStatus) -> Self {
        match value {
            FuelCoreClientTransactionStatus::PreconfirmationFailure {
                ..
            } => TransactionStatus::PreConfirmationFailed,
            FuelCoreClientTransactionStatus::PreconfirmationSuccess {
                ..
            } => TransactionStatus::PreConfirmationSuccess,
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
        let value: TransactionStatus = serde_json::from_value(json).unwrap();
        assert_eq!(value, TransactionStatus::Submitted);
    }

    #[test]
    fn test_deserialize_mixed_case() {
        let json = json!("Submitted");
        let result: Result<TransactionStatus, _> = serde_json::from_value(json);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), TransactionStatus::Submitted);

        let json = json!("SUBMITTED");
        let result: Result<TransactionStatus, _> = serde_json::from_value(json);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), TransactionStatus::Submitted);
    }

    #[test]
    fn test_deserialize_invalid() {
        let json = json!("invalid");
        let result: Result<TransactionStatus, _> = serde_json::from_value(json);
        assert!(result.is_err());
    }

    #[test]
    fn test_serialize() {
        let value = TransactionStatus::SqueezedOut;
        let serialized = serde_json::to_value(value).unwrap();
        assert_eq!(serialized, json!("squeezed_out"));
    }

    #[test]
    fn test_case_insensitive_from_str() {
        assert_eq!(
            TransactionStatus::from_str("SUBMITTED").unwrap(),
            TransactionStatus::Submitted
        );
        assert_eq!(
            TransactionStatus::from_str("submitted").unwrap(),
            TransactionStatus::Submitted
        );
        assert_eq!(
            TransactionStatus::from_str("Submitted").unwrap(),
            TransactionStatus::Submitted
        );
    }

    #[test]
    fn test_display() {
        assert_eq!(TransactionStatus::Failed.to_string(), "failed");
        assert_eq!(TransactionStatus::Submitted.to_string(), "submitted");
        assert_eq!(TransactionStatus::SqueezedOut.to_string(), "squeezed_out");
    }
}
