use std::{fmt::Display, str::FromStr};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum DbTransactionType {
    Script,
    Create,
    Mint,
    Upgrade,
    Upload,
    Blob,
}

impl<'r> sqlx::Decode<'r, sqlx::Postgres> for DbTransactionType {
    fn decode(
        value: sqlx::postgres::PgValueRef<'r>,
    ) -> Result<Self, Box<dyn std::error::Error + 'static + Send + Sync>> {
        let value = <String as sqlx::Decode<sqlx::Postgres>>::decode(value)?;
        match value.as_str() {
            "SCRIPT" => Ok(DbTransactionType::Script),
            "CREATE" => Ok(DbTransactionType::Create),
            "MINT" => Ok(DbTransactionType::Mint),
            "UPGRADE" => Ok(DbTransactionType::Upgrade),
            "UPLOAD" => Ok(DbTransactionType::Upload),
            "BLOB" => Ok(DbTransactionType::Blob),
            _ => Err(format!("Unknown DbTransactionType: {}", value).into()),
        }
    }
}

impl sqlx::Type<sqlx::Postgres> for DbTransactionType {
    fn type_info() -> sqlx::postgres::PgTypeInfo {
        sqlx::postgres::PgTypeInfo::with_name("transaction_type")
    }
}

impl sqlx::Encode<'_, sqlx::Postgres> for DbTransactionType {
    fn encode_by_ref(
        &self,
        buf: &mut sqlx::postgres::PgArgumentBuffer,
    ) -> Result<
        sqlx::encode::IsNull,
        Box<dyn std::error::Error + Send + Sync + 'static>,
    > {
        let s = match self {
            DbTransactionType::Script => "SCRIPT",
            DbTransactionType::Create => "CREATE",
            DbTransactionType::Mint => "MINT",
            DbTransactionType::Upgrade => "UPGRADE",
            DbTransactionType::Upload => "UPLOAD",
            DbTransactionType::Blob => "BLOB",
        };
        <&str as sqlx::Encode<sqlx::Postgres>>::encode_by_ref(&s, buf)
    }
}

impl FromStr for DbTransactionType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().as_str() {
            "SCRIPT" => Ok(DbTransactionType::Script),
            "CREATE" => Ok(DbTransactionType::Create),
            "MINT" => Ok(DbTransactionType::Mint),
            "UPGRADE" => Ok(DbTransactionType::Upgrade),
            "UPLOAD" => Ok(DbTransactionType::Upload),
            "BLOB" => Ok(DbTransactionType::Blob),
            _ => Err(format!("Unknown DbTransactionType: {}", s)),
        }
    }
}

impl Display for DbTransactionType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            DbTransactionType::Script => "SCRIPT",
            DbTransactionType::Create => "CREATE",
            DbTransactionType::Mint => "MINT",
            DbTransactionType::Upgrade => "UPGRADE",
            DbTransactionType::Upload => "UPLOAD",
            DbTransactionType::Blob => "BLOB",
        };
        write!(f, "{}", s)
    }
}
