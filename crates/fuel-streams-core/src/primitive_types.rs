use fuel_core_types::fuel_types;
pub use serde::{Deserialize, Serialize};

use crate::fuel_core_types::*;

/// Implements hex-formatted serialization and deserialization for a type
/// that implements Display and FromStr
macro_rules! impl_hex_serde {
    ($type:ty) => {
        impl Serialize for $type {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: serde::Serializer,
            {
                serializer.serialize_str(&self.to_string())
            }
        }

        impl<'de> Deserialize<'de> for $type {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                let s = String::deserialize(deserializer)?;
                s.parse().map_err(serde::de::Error::custom)
            }
        }
    };
}

/// Macro to generate a wrapper type for different byte-based types (including Address type).
///
/// This macro creates a new struct that wraps the specified inner type,
/// typically used for various byte-based identifiers in the Fuel ecosystem.
/// It automatically implements:
///
/// - `From<inner_type>` for easy conversion from the inner type
/// - `Display` for formatting (prefixes the output with "0x")
/// - `PartialEq` for equality comparison
/// - A `zeroed()` method to create an instance filled with zeros
///
/// # Usage
///
/// ```no_compile
/// # use fuel_streams_core::generate_byte_type_wrapper;
/// generate_byte_type_wrapper!(AddressWrapped, fuel_core_types::fuel_tx::Address);
/// ```
///
/// Where `WrapperType` is the name of the new wrapper struct to be created,
/// and `InnerType` is the type being wrapped.
macro_rules! generate_byte_type_wrapper {
    ($wrapper_type:ident, $inner_type:ty, $byte_size:expr) => {
        #[derive(Debug, Clone, PartialEq, Eq, Hash)]
        pub struct $wrapper_type(pub $inner_type);

        impl_hex_serde!($wrapper_type);

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

        impl From<[u8; $byte_size]> for $wrapper_type {
            fn from(value: [u8; $byte_size]) -> Self {
                $wrapper_type(<$inner_type>::from(value))
            }
        }

        impl From<&$inner_type> for $wrapper_type {
            fn from(value: &$inner_type) -> Self {
                $wrapper_type(*value)
            }
        }

        impl std::fmt::Display for $wrapper_type {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "0x{}", self.0)
            }
        }

        impl std::str::FromStr for $wrapper_type {
            type Err = String;

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                let s = s.strip_prefix("0x").unwrap_or(s);
                if s.len() != $byte_size {
                    panic!("Invalid length for {}", stringify!($wrapper_type));
                }
                let mut inner = <$inner_type>::zeroed();
                for (i, chunk) in s.as_bytes().chunks(2).enumerate() {
                    let byte = u8::from_str_radix(
                        std::str::from_utf8(chunk).unwrap(),
                        16,
                    )
                    .unwrap();
                    inner.as_mut()[i] = byte;
                }
                Ok($wrapper_type(inner))
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

generate_byte_type_wrapper!(Address, fuel_types::Address, 32);
generate_byte_type_wrapper!(Bytes32, fuel_types::Bytes32, 32);
generate_byte_type_wrapper!(ContractId, fuel_types::ContractId, 32);
generate_byte_type_wrapper!(AssetId, fuel_types::AssetId, 32);
generate_byte_type_wrapper!(BlobId, fuel_types::BlobId, 32);
generate_byte_type_wrapper!(Nonce, fuel_types::Nonce, 32);
generate_byte_type_wrapper!(Salt, fuel_types::Salt, 32);
generate_byte_type_wrapper!(MessageId, fuel_types::MessageId, 32);
generate_byte_type_wrapper!(BlockId, fuel_types::Bytes32, 32);
generate_byte_type_wrapper!(Signature, fuel_types::Bytes64, 64);

impl From<FuelCoreBlockId> for BlockId {
    fn from(value: FuelCoreBlockId) -> Self {
        Self(FuelCoreBytes32::from(value))
    }
}

impl From<Bytes32> for MessageId {
    fn from(value: Bytes32) -> Self {
        let bytes: [u8; 32] = value.0.into();
        MessageId::from(bytes)
    }
}
impl From<&Bytes32> for MessageId {
    fn from(value: &Bytes32) -> Self {
        value.clone().into()
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Hash)]
pub struct HexString(pub Vec<u8>);
impl_hex_serde!(HexString);

impl From<&[u8]> for HexString {
    fn from(value: &[u8]) -> Self {
        HexString(value.to_vec())
    }
}

impl std::fmt::Display for HexString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "0x{}", hex::encode(&self.0))
    }
}

impl std::str::FromStr for HexString {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.strip_prefix("0x").unwrap_or(s);
        hex::decode(s).map(HexString).map_err(|e| e.to_string())
    }
}

#[derive(
    Debug,
    Default,
    Copy,
    Clone,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    Deserialize,
    Serialize,
)]
#[serde(rename_all = "camelCase")]
pub struct TxPointer {
    block_height: FuelCoreBlockHeight,
    tx_index: u16,
}

impl From<FuelCoreTxPointer> for TxPointer {
    fn from(value: FuelCoreTxPointer) -> Self {
        Self {
            block_height: value.block_height(),
            tx_index: value.tx_index(),
        }
    }
}

#[derive(
    Debug,
    Default,
    Copy,
    Clone,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    Serialize,
    Deserialize,
)]
#[serde(rename_all = "camelCase")]
pub struct UtxoId {
    tx_id: FuelCoreTxId,
    output_index: u16,
}
impl From<FuelCoreUtxoId> for UtxoId {
    fn from(value: FuelCoreUtxoId) -> Self {
        Self::from(&value)
    }
}
impl From<&FuelCoreUtxoId> for UtxoId {
    fn from(value: &FuelCoreUtxoId) -> Self {
        Self {
            tx_id: *value.tx_id(),
            output_index: value.output_index(),
        }
    }
}

/// Macro to implement conversion from a type to `Bytes32`.
///
/// This macro creates an implementation of the `From` trait, allowing for conversion
/// from the specified type into a `Bytes32` type. It is useful when working with
/// byte-based types, such as `ContractId`, in the Fuel ecosystem.
///
/// The generated implementation allows conversion by dereferencing the input value
/// and creating a `Bytes32` type from it, making the conversion simple and efficient.
///
/// # Usage
///
/// impl_from_bytes32!(FromType);
///
///
/// Where `FromType` is the type that you want to be able to convert into a `Bytes32`.
//
/// # Notes
///
/// The macro assumes that the type being converted can be dereferenced into a byte-based type
/// compatible with the `Bytes32` structure.
macro_rules! impl_from_bytes32 {
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

impl_from_bytes32!(fuel_types::ContractId);
impl_from_bytes32!(fuel_types::AssetId);
impl_from_bytes32!(fuel_types::Address);
