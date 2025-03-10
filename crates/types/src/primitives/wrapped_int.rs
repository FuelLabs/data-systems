use crate::impl_utoipa_for_integer_wrapper;

#[macro_export]
macro_rules! impl_conversions {
    ($name:ident, $inner_type:ty, $($t:ty),*) => {
        $(
            impl From<$t> for $name {
                fn from(value: $t) -> Self {
                    $name(value as $inner_type)
                }
            }

            impl From<$name> for $t {
                fn from(value: $name) -> Self {
                    value.0 as $t
                }
            }
        )*

        impl From<Option<$inner_type>> for $name {
            fn from(value: Option<$inner_type>) -> Self {
                match value {
                    Some(v) => Self(v),
                    None => Self::default(),
                }
            }
        }

        impl From<$name> for Option<$inner_type> {
            fn from(value: $name) -> Self {
                Some(value.0)
            }
        }
    };
}

#[macro_export]
macro_rules! declare_integer_wrapper {
    ($name:ident, $inner_type:ty, $error:ty) => {
        #[derive(Debug, Clone, Eq, PartialEq, Hash, Copy)]
        pub struct $name($inner_type);

        impl $name {
            pub fn new<T: Into<$inner_type>>(value: T) -> Self {
                Self(value.into())
            }

            pub fn into_inner(&self) -> $inner_type {
                self.0
            }
        }

        impl Default for $name {
            fn default() -> Self {
                0.into()
            }
        }

        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}", self.0)
            }
        }

        impl std::ops::Deref for $name {
            type Target = $inner_type;
            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }

        impl AsRef<$inner_type> for $name {
            fn as_ref(&self) -> &$inner_type {
                &self.0
            }
        }

        impl PartialOrd for $name {
            fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
                Some(self.cmp(other))
            }
        }

        impl Ord for $name {
            fn cmp(&self, other: &Self) -> std::cmp::Ordering {
                self.into_inner().cmp(&other.into_inner())
            }
        }

        impl TryFrom<&str> for $name {
            type Error = $error;
            fn try_from(s: &str) -> Result<Self, Self::Error> {
                let value = s
                    .parse::<$inner_type>()
                    .map_err(|_| <$error>::InvalidFormat(s.to_string()))?;
                Ok($name(value))
            }
        }

        impl TryFrom<String> for $name {
            type Error = $error;
            fn try_from(s: String) -> Result<Self, Self::Error> {
                s.as_str().try_into()
            }
        }

        impl std::str::FromStr for $name {
            type Err = $error;
            fn from_str(s: &str) -> Result<Self, Self::Err> {
                s.try_into()
            }
        }

        impl serde::Serialize for $name {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: serde::Serializer,
            {
                serializer.serialize_str(&self.0.to_string())
            }
        }

        impl<'de> serde::Deserialize<'de> for $name {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct IntegerVisitor;

                impl<'de> serde::de::Visitor<'de> for IntegerVisitor {
                    type Value = $name;

                    fn expecting(
                        &self,
                        formatter: &mut std::fmt::Formatter,
                    ) -> std::fmt::Result {
                        formatter.write_str("a string, number, or null")
                    }

                    fn visit_str<E>(self, value: &str) -> Result<$name, E>
                    where
                        E: serde::de::Error,
                    {
                        value
                            .parse::<$inner_type>()
                            .map($name)
                            .map_err(serde::de::Error::custom)
                    }

                    fn visit_u32<E>(self, value: u32) -> Result<$name, E>
                    where
                        E: serde::de::Error,
                    {
                        Ok($name(value as $inner_type))
                    }

                    fn visit_i32<E>(self, value: i32) -> Result<$name, E>
                    where
                        E: serde::de::Error,
                    {
                        Ok($name(value as $inner_type))
                    }

                    fn visit_u64<E>(self, value: u64) -> Result<$name, E>
                    where
                        E: serde::de::Error,
                    {
                        Ok($name(value as $inner_type))
                    }

                    fn visit_i64<E>(self, value: i64) -> Result<$name, E>
                    where
                        E: serde::de::Error,
                    {
                        Ok($name(value as $inner_type))
                    }

                    fn visit_none<E>(self) -> Result<$name, E>
                    where
                        E: serde::de::Error,
                    {
                        Ok($name::default())
                    }

                    fn visit_unit<E>(self) -> Result<$name, E>
                    where
                        E: serde::de::Error,
                    {
                        Ok($name::default())
                    }
                }

                deserializer.deserialize_any(IntegerVisitor)
            }
        }

        $crate::impl_conversions!($name, $inner_type, u32, i32, u64, i64);

        impl sqlx::Type<sqlx::Postgres> for $name {
            fn type_info() -> sqlx::postgres::PgTypeInfo {
                match std::any::TypeId::of::<$inner_type>() {
                    id if id == std::any::TypeId::of::<u32>() => {
                        <i32 as sqlx::Type<sqlx::Postgres>>::type_info()
                    }
                    id if id == std::any::TypeId::of::<u64>() => {
                        <i64 as sqlx::Type<sqlx::Postgres>>::type_info()
                    }
                    _ => {
                        panic!(
                            "Unsupported inner type: {:?}",
                            std::any::TypeId::of::<$inner_type>()
                        );
                    }
                }
            }
        }

        impl<'r> sqlx::Decode<'r, sqlx::Postgres> for $name {
            fn decode(
                value: sqlx::postgres::PgValueRef<'r>,
            ) -> Result<Self, sqlx::error::BoxDynError> {
                match std::any::TypeId::of::<$inner_type>() {
                    id if id == std::any::TypeId::of::<u32>() => {
                        let value =
                            <i32 as sqlx::Decode<sqlx::Postgres>>::decode(
                                value,
                            )?;
                        Ok($name::new(value as $inner_type))
                    }
                    id if id == std::any::TypeId::of::<u64>() => {
                        let value =
                            <i64 as sqlx::Decode<sqlx::Postgres>>::decode(
                                value,
                            )?;
                        Ok($name::new(value as $inner_type))
                    }
                    _ => {
                        panic!(
                            "Unsupported inner type: {:?}",
                            std::any::TypeId::of::<$inner_type>()
                        );
                    }
                }
            }
        }

        impl sqlx::Encode<'_, sqlx::Postgres> for $name {
            fn encode_by_ref(
                &self,
                buf: &mut sqlx::postgres::PgArgumentBuffer,
            ) -> Result<sqlx::encode::IsNull, sqlx::error::BoxDynError> {
                match std::any::TypeId::of::<$inner_type>() {
                    id if id == std::any::TypeId::of::<u32>() => {
                        <i32 as sqlx::Encode<sqlx::Postgres>>::encode_by_ref(
                            &(self.0 as i32),
                            buf,
                        )
                    }
                    id if id == std::any::TypeId::of::<u64>() => {
                        <i64 as sqlx::Encode<sqlx::Postgres>>::encode_by_ref(
                            &(self.0 as i64),
                            buf,
                        )
                    }
                    _ => {
                        panic!(
                            "Unsupported inner type: {:?}",
                            std::any::TypeId::of::<$inner_type>()
                        );
                    }
                }
            }
        }
    };
}

#[derive(thiserror::Error, Debug)]
pub enum WrappedIntError {
    #[error("Invalid format: {0}")]
    InvalidFormat(String),
}

declare_integer_wrapper!(WrappedU32, u32, WrappedIntError);
declare_integer_wrapper!(WrappedU64, u64, WrappedIntError);

impl_utoipa_for_integer_wrapper!(
    WrappedU32,
    "Wrapped u32 in the blockchain",
    0,
    u32::MAX as usize
);

impl_utoipa_for_integer_wrapper!(
    WrappedU64,
    "Wrapped u64 in the blockchain",
    0,
    u64::MAX as usize
);
