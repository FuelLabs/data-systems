use std::{fmt, str::FromStr};

use serde::{Deserialize, Serialize};

use crate::fuel_core::{
    FuelCoreClientTransactionStatus,
    FuelCoreTransactionStatus,
};

#[derive(
    Debug, Default, Clone, PartialEq, Serialize, Deserialize, utoipa::ToSchema,
)]
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

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Serialize,
    Deserialize,
    derive_more::Display,
    derive_more::FromStr,
)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum DbTransactionStatus {
    #[display("FAILED")]
    Failed,
    #[display("SUBMITTED")]
    Submitted,
    #[display("SQUEEZED_OUT")]
    SqueezedOut,
    #[display("SUCCESS")]
    Success,
    #[display("NONE")]
    None,
}

impl From<TransactionStatus> for DbTransactionStatus {
    fn from(value: TransactionStatus) -> Self {
        match value {
            TransactionStatus::Failed => DbTransactionStatus::Failed,
            TransactionStatus::Submitted => DbTransactionStatus::Submitted,
            TransactionStatus::SqueezedOut => DbTransactionStatus::SqueezedOut,
            TransactionStatus::Success => DbTransactionStatus::Success,
            TransactionStatus::None => DbTransactionStatus::None,
        }
    }
}

impl<'r> sqlx::Decode<'r, sqlx::Postgres> for DbTransactionStatus {
    fn decode(
        value: sqlx::postgres::PgValueRef<'r>,
    ) -> Result<Self, Box<dyn std::error::Error + 'static + Send + Sync>> {
        let value = <String as sqlx::Decode<sqlx::Postgres>>::decode(value)?;
        match value.as_str() {
            "FAILED" => Ok(DbTransactionStatus::Failed),
            "SUBMITTED" => Ok(DbTransactionStatus::Submitted),
            "SQUEEZED_OUT" => Ok(DbTransactionStatus::SqueezedOut),
            "SUCCESS" => Ok(DbTransactionStatus::Success),
            "NONE" => Ok(DbTransactionStatus::None),
            _ => Err(format!("Unknown DbTransactionStatus: {}", value).into()),
        }
    }
}

impl sqlx::Type<sqlx::Postgres> for DbTransactionStatus {
    fn type_info() -> sqlx::postgres::PgTypeInfo {
        sqlx::postgres::PgTypeInfo::with_name("transaction_status")
    }
}

impl sqlx::Encode<'_, sqlx::Postgres> for DbTransactionStatus {
    fn encode_by_ref(
        &self,
        buf: &mut sqlx::postgres::PgArgumentBuffer,
    ) -> Result<
        sqlx::encode::IsNull,
        Box<dyn std::error::Error + Send + Sync + 'static>,
    > {
        let s = match self {
            DbTransactionStatus::Failed => "FAILED",
            DbTransactionStatus::Submitted => "SUBMITTED",
            DbTransactionStatus::SqueezedOut => "SQUEEZED_OUT",
            DbTransactionStatus::Success => "SUCCESS",
            DbTransactionStatus::None => "NONE",
        };
        <&str as sqlx::Encode<sqlx::Postgres>>::encode_by_ref(&s, buf)
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
