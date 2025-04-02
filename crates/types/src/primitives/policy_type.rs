use serde::{Deserialize, Serialize};

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Serialize,
    Deserialize,
    derive_more::FromStr,
    derive_more::Display,
)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum DbPolicyType {
    #[display("TIP")]
    Tip,
    #[display("WITNESS_LIMIT")]
    WitnessLimit,
    #[display("MATURITY")]
    Maturity,
    #[display("MAX_FEE")]
    MaxFee,
}

impl<'r> sqlx::Decode<'r, sqlx::Postgres> for DbPolicyType {
    fn decode(
        value: sqlx::postgres::PgValueRef<'r>,
    ) -> Result<Self, Box<dyn std::error::Error + 'static + Send + Sync>> {
        let value = <String as sqlx::Decode<sqlx::Postgres>>::decode(value)?;
        match value.as_str() {
            "TIP" => Ok(DbPolicyType::Tip),
            "WITNESS_LIMIT" => Ok(DbPolicyType::WitnessLimit),
            "MATURITY" => Ok(DbPolicyType::Maturity),
            "MAX_FEE" => Ok(DbPolicyType::MaxFee),
            _ => Err(format!("Unknown DbPolicyType: {}", value).into()),
        }
    }
}

impl sqlx::Type<sqlx::Postgres> for DbPolicyType {
    fn type_info() -> sqlx::postgres::PgTypeInfo {
        sqlx::postgres::PgTypeInfo::with_name("policy_type")
    }
}

impl sqlx::Encode<'_, sqlx::Postgres> for DbPolicyType {
    fn encode_by_ref(
        &self,
        buf: &mut sqlx::postgres::PgArgumentBuffer,
    ) -> Result<
        sqlx::encode::IsNull,
        Box<dyn std::error::Error + Send + Sync + 'static>,
    > {
        let s = match self {
            DbPolicyType::Tip => "TIP",
            DbPolicyType::WitnessLimit => "WITNESS_LIMIT",
            DbPolicyType::Maturity => "MATURITY",
            DbPolicyType::MaxFee => "MAX_FEE",
        };
        <&str as sqlx::Encode<sqlx::Postgres>>::encode_by_ref(&s, buf)
    }
}
