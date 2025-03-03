use std::fmt;

use serde::{Deserialize, Serialize};

use crate::api_key::ApiKeyError;

#[derive(
    Debug,
    Serialize,
    Deserialize,
    Clone,
    Eq,
    PartialEq,
    Default,
    strum::EnumIter,
)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ApiKeyRoleName {
    Admin,
    Amm,
    Builder,
    #[default]
    WebClient,
}

impl ApiKeyRoleName {
    pub fn as_str(&self) -> &str {
        match self {
            ApiKeyRoleName::Admin => "ADMIN",
            ApiKeyRoleName::Amm => "AMM",
            ApiKeyRoleName::Builder => "BUILDER",
            ApiKeyRoleName::WebClient => "WEB_CLIENT",
        }
    }

    pub fn is_admin(&self) -> bool {
        matches!(self, ApiKeyRoleName::Admin)
    }

    pub fn is_amm(&self) -> bool {
        matches!(self, ApiKeyRoleName::Amm)
    }

    pub fn is_builder(&self) -> bool {
        matches!(self, ApiKeyRoleName::Builder)
    }

    pub fn is_web_client(&self) -> bool {
        matches!(self, ApiKeyRoleName::WebClient)
    }
}

impl fmt::Display for ApiKeyRoleName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl TryFrom<&str> for ApiKeyRoleName {
    type Error = ApiKeyError;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "ADMIN" => Ok(ApiKeyRoleName::Admin),
            "AMM" => Ok(ApiKeyRoleName::Amm),
            "BUILDER" => Ok(ApiKeyRoleName::Builder),
            "WEB_CLIENT" => Ok(ApiKeyRoleName::WebClient),
            _ => Err(ApiKeyError::RolePermission(value.to_string())),
        }
    }
}

impl sqlx::Type<sqlx::Postgres> for ApiKeyRoleName {
    fn type_info() -> sqlx::postgres::PgTypeInfo {
        sqlx::postgres::PgTypeInfo::with_name("api_role")
    }
}

impl<'r> sqlx::Decode<'r, sqlx::Postgres> for ApiKeyRoleName {
    fn decode(
        value: sqlx::postgres::PgValueRef<'r>,
    ) -> Result<Self, sqlx::error::BoxDynError> {
        let value = <&str as sqlx::Decode<sqlx::Postgres>>::decode(value)?;
        ApiKeyRoleName::try_from(value).map_err(sqlx::error::BoxDynError::from)
    }
}

impl sqlx::Encode<'_, sqlx::Postgres> for ApiKeyRoleName {
    fn encode_by_ref(
        &self,
        buf: &mut sqlx::postgres::PgArgumentBuffer,
    ) -> Result<sqlx::encode::IsNull, sqlx::error::BoxDynError> {
        <&str as sqlx::Encode<sqlx::Postgres>>::encode_by_ref(
            &self.as_str(),
            buf,
        )
    }
}
