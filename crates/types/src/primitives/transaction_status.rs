use std::{fmt::Display, str::FromStr};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum DbTransactionStatus {
    Failed,
    Submitted,
    SqueezedOut,
    Success,
    None,
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

impl FromStr for DbTransactionStatus {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().as_str() {
            "FAILED" => Ok(DbTransactionStatus::Failed),
            "SUBMITTED" => Ok(DbTransactionStatus::Submitted),
            "SQUEEZED_OUT" => Ok(DbTransactionStatus::SqueezedOut),
            "SUCCESS" => Ok(DbTransactionStatus::Success),
            "NONE" => Ok(DbTransactionStatus::None),
            _ => Err(format!("Unknown DbTransactionStatus: {}", s)),
        }
    }
}

impl Display for DbTransactionStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            DbTransactionStatus::Failed => "FAILED",
            DbTransactionStatus::Submitted => "SUBMITTED",
            DbTransactionStatus::SqueezedOut => "SQUEEZED_OUT",
            DbTransactionStatus::Success => "SUCCESS",
            DbTransactionStatus::None => "NONE",
        };
        write!(f, "{}", s)
    }
}
