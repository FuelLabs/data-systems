use std::{fmt::Display, str::FromStr};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum DbConsensusType {
    Genesis,
    PoaConsensus,
}

impl<'r> sqlx::Decode<'r, sqlx::Postgres> for DbConsensusType {
    fn decode(
        value: sqlx::postgres::PgValueRef<'r>,
    ) -> Result<Self, Box<dyn std::error::Error + 'static + Send + Sync>> {
        let value = <String as sqlx::Decode<sqlx::Postgres>>::decode(value)?;
        match value.as_str() {
            "GENESIS" => Ok(DbConsensusType::Genesis),
            "POA_CONSENSUS" => Ok(DbConsensusType::PoaConsensus),
            _ => Err(format!("Unknown ConsensusType: {}", value).into()),
        }
    }
}

impl sqlx::Type<sqlx::Postgres> for DbConsensusType {
    fn type_info() -> sqlx::postgres::PgTypeInfo {
        sqlx::postgres::PgTypeInfo::with_name("consensus_type")
    }
}

impl sqlx::Encode<'_, sqlx::Postgres> for DbConsensusType {
    fn encode_by_ref(
        &self,
        buf: &mut sqlx::postgres::PgArgumentBuffer,
    ) -> Result<
        sqlx::encode::IsNull,
        Box<dyn std::error::Error + Send + Sync + 'static>,
    > {
        let s = match self {
            DbConsensusType::Genesis => "GENESIS",
            DbConsensusType::PoaConsensus => "POA_CONSENSUS",
        };
        <&str as sqlx::Encode<sqlx::Postgres>>::encode_by_ref(&s, buf)
    }
}

impl FromStr for DbConsensusType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().as_str() {
            "GENESIS" => Ok(DbConsensusType::Genesis),
            "POA_CONSENSUS" => Ok(DbConsensusType::PoaConsensus),
            _ => Err(format!("Unknown DbConsensusType: {}", s)),
        }
    }
}

impl Display for DbConsensusType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            DbConsensusType::Genesis => "GENESIS",
            DbConsensusType::PoaConsensus => "POA_CONSENSUS",
        };
        write!(f, "{}", s)
    }
}
