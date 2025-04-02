use serde::{Deserialize, Serialize};

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
pub enum DbTransactionAccountType {
    #[display("CONTRACT")]
    Contract,
    #[display("ADDRESS")]
    Address,
    #[display("PREDICATE")]
    Predicate,
    #[display("SCRIPT")]
    Script,
}

impl<'r> sqlx::Decode<'r, sqlx::Postgres> for DbTransactionAccountType {
    fn decode(
        value: sqlx::postgres::PgValueRef<'r>,
    ) -> Result<Self, Box<dyn std::error::Error + 'static + Send + Sync>> {
        let value = <String as sqlx::Decode<sqlx::Postgres>>::decode(value)?;
        match value.as_str() {
            "CONTRACT" => Ok(DbTransactionAccountType::Contract),
            "ADDRESS" => Ok(DbTransactionAccountType::Address),
            "PREDICATE" => Ok(DbTransactionAccountType::Predicate),
            "SCRIPT" => Ok(DbTransactionAccountType::Script),
            _ => {
                Err(format!("Unknown DbTransactionAccountType: {}", value)
                    .into())
            }
        }
    }
}

impl sqlx::Type<sqlx::Postgres> for DbTransactionAccountType {
    fn type_info() -> sqlx::postgres::PgTypeInfo {
        sqlx::postgres::PgTypeInfo::with_name("transaction_account_type")
    }
}

impl sqlx::Encode<'_, sqlx::Postgres> for DbTransactionAccountType {
    fn encode_by_ref(
        &self,
        buf: &mut sqlx::postgres::PgArgumentBuffer,
    ) -> Result<
        sqlx::encode::IsNull,
        Box<dyn std::error::Error + Send + Sync + 'static>,
    > {
        let s = match self {
            DbTransactionAccountType::Contract => "CONTRACT",
            DbTransactionAccountType::Address => "ADDRESS",
            DbTransactionAccountType::Predicate => "PREDICATE",
            DbTransactionAccountType::Script => "SCRIPT",
        };
        <&str as sqlx::Encode<sqlx::Postgres>>::encode_by_ref(&s, buf)
    }
}
