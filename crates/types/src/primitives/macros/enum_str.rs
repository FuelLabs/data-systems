#[macro_export]
macro_rules! impl_enum_string_serialization {
    ($name:ident, $db_name:expr) => {
        impl<'de> serde::Deserialize<'de> for $name {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                let s = String::deserialize(deserializer)?;
                $name::try_from(s.as_str()).map_err(serde::de::Error::custom)
            }
        }

        impl<'r> sqlx::Decode<'r, sqlx::Postgres> for $name {
            fn decode(
                value: sqlx::postgres::PgValueRef<'r>,
            ) -> Result<Self, Box<dyn std::error::Error + 'static + Send + Sync>> {
                let value = <String as sqlx::Decode<sqlx::Postgres>>::decode(value)?;
                $name::try_from(value.as_str()).map_err(|e| e.into())
            }
        }

        impl sqlx::Type<sqlx::Postgres> for $name {
            fn type_info() -> sqlx::postgres::PgTypeInfo {
                sqlx::postgres::PgTypeInfo::with_name($db_name)
            }
        }

        impl sqlx::Encode<'_, sqlx::Postgres> for $name {
            fn encode_by_ref(
                &self,
                buf: &mut sqlx::postgres::PgArgumentBuffer,
            ) -> Result<
                sqlx::encode::IsNull,
                Box<dyn std::error::Error + Send + Sync + 'static>,
            > {
                <&str as sqlx::Encode<sqlx::Postgres>>::encode_by_ref(
                    &self.to_string().as_str(),
                    buf,
                )
            }
        }

        impl FromStr for $name {
            type Err = String;
            fn from_str(s: &str) -> Result<Self, Self::Err> {
                $name::try_from(s)
            }
        }
    };
}
