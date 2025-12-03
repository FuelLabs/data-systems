/// Common wrapper type macro that implements basic traits and conversions
#[macro_export]
macro_rules! common_wrapper_type {
    ($wrapper_type:ident, $inner_type:ty) => {
        #[derive(Debug, Clone, PartialEq, Eq, Hash)]
        pub struct $wrapper_type(pub $inner_type);

        impl $wrapper_type {
            pub fn into_inner(self) -> $inner_type {
                self.0
            }

            pub fn len(&self) -> usize {
                self.0.len()
            }

            pub fn is_empty(&self) -> bool {
                self.len() == 0
            }
        }

        impl From<$inner_type> for $wrapper_type {
            fn from(value: $inner_type) -> Self {
                $wrapper_type(value)
            }
        }

        impl From<$wrapper_type> for $inner_type {
            fn from(value: $wrapper_type) -> Self {
                value.0
            }
        }

        impl From<&$inner_type> for $wrapper_type {
            fn from(value: &$inner_type) -> Self {
                $wrapper_type(value.clone())
            }
        }

        impl std::fmt::Display for $wrapper_type {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "0x{}", self.0)
            }
        }

        impl From<&str> for $wrapper_type {
            fn from(s: &str) -> Self {
                s.parse().unwrap_or_else(|e| {
                    panic!("Failed to parse {}: {}", stringify!($wrapper_type), e)
                })
            }
        }

        impl $wrapper_type {
            pub fn zeroed() -> Self {
                $wrapper_type(<$inner_type>::zeroed())
            }

            pub fn new(inner: $inner_type) -> Self {
                $wrapper_type(inner)
            }
        }

        impl AsRef<$inner_type> for $wrapper_type {
            fn as_ref(&self) -> &$inner_type {
                &self.0
            }
        }

        impl Default for $wrapper_type {
            fn default() -> Self {
                $wrapper_type(<$inner_type>::zeroed())
            }
        }
    };
}

/// Macro for generating byte type wrappers with optional byte size specification
#[macro_export]
macro_rules! generate_byte_type_wrapper {
    // Pattern with byte_size specified
    ($wrapper_type:ident, $inner_type:ty, $byte_size:expr) => {
        $crate::common_wrapper_type!($wrapper_type, $inner_type);

        impl $wrapper_type {
            pub fn random() -> Self {
                use rand::prelude::*;
                let mut rng = rand::rng();
                let bytes: [u8; $byte_size] = rng.random();
                Self(<$inner_type>::from(bytes))
            }
        }

        // Custom serialization
        impl serde::Serialize for $wrapper_type {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: serde::Serializer,
            {
                if serializer.is_human_readable() {
                    serializer.serialize_str(&format!("0x{}", self.0))
                } else {
                    self.0.serialize(serializer)
                }
            }
        }

        // Custom deserialization using FromStr
        impl<'de> serde::Deserialize<'de> for $wrapper_type {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                if deserializer.is_human_readable() {
                    let s = String::deserialize(deserializer)?;
                    s.parse().map_err(serde::de::Error::custom)
                } else {
                    Ok($wrapper_type(<$inner_type>::deserialize(deserializer)?))
                }
            }
        }

        impl From<[u8; $byte_size]> for $wrapper_type {
            fn from(value: [u8; $byte_size]) -> Self {
                $wrapper_type(<$inner_type>::from(value))
            }
        }

        impl std::str::FromStr for $wrapper_type {
            type Err = String;

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                let s = s.strip_prefix("0x").unwrap_or(s);
                if s.len() != std::mem::size_of::<$inner_type>() * 2 {
                    return Err(format!(
                        "Invalid length for {}, expected {} characters",
                        stringify!($wrapper_type),
                        std::mem::size_of::<$inner_type>() * 2
                    ));
                }
                let bytes = hex::decode(s)
                    .map_err(|e| format!("Failed to decode hex string: {}", e))?;
                let array: [u8; $byte_size] = bytes
                    .try_into()
                    .map_err(|_| "Invalid byte length".to_string())?;
                Ok($wrapper_type(<$inner_type>::from(array)))
            }
        }

        impl sqlx::Type<sqlx::Postgres> for $wrapper_type {
            fn type_info() -> sqlx::postgres::PgTypeInfo {
                <String as sqlx::Type<sqlx::Postgres>>::type_info()
            }

            fn compatible(ty: &sqlx::postgres::PgTypeInfo) -> bool {
                *ty == sqlx::postgres::PgTypeInfo::with_name("TEXT")
            }
        }

        impl<'r> sqlx::Decode<'r, sqlx::Postgres> for $wrapper_type {
            fn decode(
                value: sqlx::postgres::PgValueRef<'r>,
            ) -> Result<Self, sqlx::error::BoxDynError> {
                let hex_str = <&str as sqlx::Decode<sqlx::Postgres>>::decode(value)?;
                let s = hex_str.strip_prefix("0x").ok_or("Missing 0x prefix")?;
                let bytes = hex::decode(s)?;
                let array: [u8; $byte_size] =
                    bytes.try_into().map_err(|_| "Invalid byte length")?;
                Ok($wrapper_type(<$inner_type>::from(array)))
            }
        }

        impl sqlx::Encode<'_, sqlx::Postgres> for $wrapper_type {
            fn encode_by_ref(
                &self,
                buf: &mut sqlx::postgres::PgArgumentBuffer,
            ) -> Result<sqlx::encode::IsNull, sqlx::error::BoxDynError> {
                let hex_str = format!("0x{}", hex::encode(self.0.as_ref()));
                sqlx::Encode::<sqlx::Postgres>::encode(&hex_str, buf)
            }
        }
    };

    // Pattern without byte_size
    ($wrapper_type:ident, $inner_type:ty) => {
        $crate::common_wrapper_type!($wrapper_type, $inner_type);

        impl $wrapper_type {
            pub fn random() -> Self {
                use rand::prelude::*;
                let mut rng = rand::rng();
                let bytes: [u8; 64] = rng.random();
                Self(<$inner_type>::from(bytes.to_vec()))
            }
        }

        impl serde::Serialize for $wrapper_type {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: serde::Serializer,
            {
                if serializer.is_human_readable() {
                    serializer.serialize_str(&format!("0x{}", self.0))
                } else {
                    self.0.serialize(serializer)
                }
            }
        }

        impl<'de> serde::Deserialize<'de> for $wrapper_type {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                use serde::Deserialize;
                struct WrapperVisitor;

                impl<'de> serde::de::Visitor<'de> for WrapperVisitor {
                    type Value = $wrapper_type;

                    fn expecting(
                        &self,
                        formatter: &mut std::fmt::Formatter,
                    ) -> std::fmt::Result {
                        write!(formatter, "a string, bytes, sequence, or null")
                    }

                    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
                    where
                        E: serde::de::Error,
                    {
                        value.parse().map_err(serde::de::Error::custom)
                    }

                    fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
                    where
                        E: serde::de::Error,
                    {
                        Ok($wrapper_type(<$inner_type>::from(v.to_vec())))
                    }

                    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
                    where
                        A: serde::de::SeqAccess<'de>,
                    {
                        let mut bytes = Vec::new();
                        while let Some(byte) = seq.next_element()? {
                            bytes.push(byte);
                        }
                        Ok($wrapper_type(<$inner_type>::from(bytes)))
                    }

                    fn visit_none<E>(self) -> Result<Self::Value, E>
                    where
                        E: serde::de::Error,
                    {
                        Ok($wrapper_type::zeroed())
                    }

                    fn visit_some<D>(
                        self,
                        deserializer: D,
                    ) -> Result<Self::Value, D::Error>
                    where
                        D: serde::Deserializer<'de>,
                    {
                        $wrapper_type::deserialize(deserializer)
                    }
                }

                if deserializer.is_human_readable() {
                    deserializer.deserialize_any(WrapperVisitor)
                } else {
                    Ok($wrapper_type(<$inner_type>::deserialize(deserializer)?))
                }
            }
        }

        impl From<Vec<u8>> for $wrapper_type {
            fn from(value: Vec<u8>) -> Self {
                $wrapper_type(<$inner_type>::from(value))
            }
        }

        impl std::str::FromStr for $wrapper_type {
            type Err = String;
            fn from_str(s: &str) -> Result<Self, Self::Err> {
                let s = s.strip_prefix("0x").unwrap_or(s);
                let bytes = hex::decode(s)
                    .map_err(|e| format!("Failed to decode hex string: {}", e))?;
                Ok($wrapper_type(bytes.into()))
            }
        }
    };
}

/// Macro for implementing Bytes32 conversions
#[macro_export]
macro_rules! impl_bytes32_to_type {
    ($type:ty) => {
        impl From<Bytes32> for $type {
            fn from(value: Bytes32) -> Self {
                let bytes: [u8; 32] = value.0.into();
                <$type>::from(bytes)
            }
        }
        impl From<&Bytes32> for $type {
            fn from(value: &Bytes32) -> Self {
                value.clone().into()
            }
        }
        impl From<$type> for Bytes32 {
            fn from(value: $type) -> Self {
                let bytes: [u8; 32] = value.0.into();
                Bytes32::from(bytes)
            }
        }
        impl From<&$type> for Bytes32 {
            fn from(value: &$type) -> Self {
                value.clone().into()
            }
        }
    };
}

/// Macro for implementing From<T> for Bytes32
#[macro_export]
macro_rules! impl_from_type_to_bytes32 {
    ($from_type:ty) => {
        impl From<$from_type> for Bytes32 {
            fn from(value: $from_type) -> Self {
                Bytes32(fuel_core_types::fuel_tx::Bytes32::from(*value))
            }
        }
        impl From<&$from_type> for Bytes32 {
            fn from(value: &$from_type) -> Self {
                (*value).into()
            }
        }
    };
}
