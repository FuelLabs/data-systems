/// Common wrapper type macro that implements basic traits and conversions
#[macro_export]
macro_rules! common_wrapper_type {
    ($wrapper_type:ident, $inner_type:ty) => {
        #[derive(Debug, Clone, PartialEq, Eq, Hash)]
        pub struct $wrapper_type(pub $inner_type);

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
                    panic!(
                        "Failed to parse {}: {}",
                        stringify!($wrapper_type),
                        e
                    )
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

        impl $wrapper_type {
            pub fn into_inner(self) -> $inner_type {
                self.0
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
                let bytes = hex::decode(s).map_err(|e| {
                    format!("Failed to decode hex string: {}", e)
                })?;
                let array: [u8; $byte_size] = bytes
                    .try_into()
                    .map_err(|_| "Invalid byte length".to_string())?;
                Ok($wrapper_type(<$inner_type>::from(array)))
            }
        }
    };

    // Pattern without byte_size
    ($wrapper_type:ident, $inner_type:ty) => {
        $crate::common_wrapper_type!($wrapper_type, $inner_type);

        impl From<Vec<u8>> for $wrapper_type {
            fn from(value: Vec<u8>) -> Self {
                $wrapper_type(<$inner_type>::from(value))
            }
        }

        impl std::str::FromStr for $wrapper_type {
            type Err = String;
            fn from_str(s: &str) -> Result<Self, Self::Err> {
                let s = s.strip_prefix("0x").unwrap_or(s);
                let bytes = hex::decode(s).map_err(|e| {
                    format!("Failed to decode hex string: {}", e)
                })?;
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
