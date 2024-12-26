use fuel_core_types::{
    fuel_asm::RawInstruction,
    fuel_tx::PanicReason,
    fuel_types,
};
pub use serde::{Deserialize, Serialize};

use crate::fuel_core_types::*;

#[derive(
    Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Default,
)]
pub struct LongBytes(pub Vec<u8>);

impl LongBytes {
    pub fn zeroed() -> Self {
        Self(vec![])
    }
}
impl AsRef<[u8]> for LongBytes {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}
impl AsMut<[u8]> for LongBytes {
    fn as_mut(&mut self) -> &mut [u8] {
        &mut self.0
    }
}
impl std::fmt::Display for LongBytes {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "0x{}", hex::encode(&self.0))
    }
}
impl From<Vec<u8>> for LongBytes {
    fn from(value: Vec<u8>) -> Self {
        Self(value)
    }
}
impl From<&[u8]> for LongBytes {
    fn from(value: &[u8]) -> Self {
        Self(value.to_vec())
    }
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
    // Pattern with byte_size specified
    ($wrapper_type:ident, $inner_type:ty, $byte_size:expr) => {
        generate_byte_type_wrapper!($wrapper_type, $inner_type);

        impl From<[u8; $byte_size]> for $wrapper_type {
            fn from(value: [u8; $byte_size]) -> Self {
                $wrapper_type(<$inner_type>::from(value))
            }
        }
    };

    // Pattern without byte_size
    ($wrapper_type:ident, $inner_type:ty) => {
        #[derive(Debug, Clone, PartialEq, Eq, Hash)]
        pub struct $wrapper_type(pub $inner_type);

        // Custom serialization
        impl Serialize for $wrapper_type {
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
        impl<'de> Deserialize<'de> for $wrapper_type {
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

        impl std::str::FromStr for $wrapper_type {
            type Err = String;
            fn from_str(s: &str) -> Result<Self, Self::Err> {
                let s = s.strip_prefix("0x").unwrap_or(s);
                let expected_len = std::mem::size_of::<$inner_type>() * 2;
                if s.len() != expected_len {
                    return Err(format!(
                        "Invalid length for {}: expected {} characters, got {}",
                        stringify!($wrapper_type),
                        expected_len,
                        s.len()
                    ));
                }

                let mut inner = <$inner_type>::zeroed();
                let bytes = hex::decode(s)
                    .map_err(|e| format!("Failed to decode hex string: {}", e))?;

                if bytes.len() != std::mem::size_of::<$inner_type>() {
                    return Err(format!(
                        "Invalid decoded length for {}: expected {} bytes, got {}",
                        stringify!($wrapper_type),
                        std::mem::size_of::<$inner_type>(),
                        bytes.len()
                    ));
                }

                inner.as_mut().copy_from_slice(&bytes);
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
generate_byte_type_wrapper!(TxId, fuel_types::TxId, 32);
generate_byte_type_wrapper!(HexData, LongBytes);

/// Implements bidirectional conversions between `Bytes32` and a given type.
///
/// This macro generates implementations of the `From` trait to convert:
/// - From `Bytes32` to the target type
/// - From a reference to `Bytes32` to the target type
/// - From the target type to `Bytes32`
/// - From a reference of the target type to `Bytes32`
///
/// The target type must be a 32-byte type that can be converted to/from `[u8; 32]`.
///
/// # Example
/// ```ignore
/// impl_bytes32_conversions!(ContractId);
/// ```
macro_rules! impl_bytes32_conversions {
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

impl_bytes32_conversions!(MessageId);
impl_bytes32_conversions!(ContractId);
impl_bytes32_conversions!(AssetId);
impl_bytes32_conversions!(Address);
impl_bytes32_conversions!(BlobId);
impl_bytes32_conversions!(Nonce);
impl_bytes32_conversions!(Salt);
impl_bytes32_conversions!(BlockId);
impl_bytes32_conversions!(TxId);

impl From<FuelCoreBlockId> for BlockId {
    fn from(value: FuelCoreBlockId) -> Self {
        Self(FuelCoreBytes32::from(value))
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
    Debug, Default, Clone, PartialEq, Eq, Hash, Serialize, Deserialize,
)]
pub struct UtxoId {
    pub tx_id: Bytes32,
    pub output_index: u16,
}
impl From<&UtxoId> for HexData {
    fn from(value: &UtxoId) -> Self {
        value.to_owned().into()
    }
}
impl From<FuelCoreUtxoId> for UtxoId {
    fn from(value: FuelCoreUtxoId) -> Self {
        Self::from(&value)
    }
}
impl From<&FuelCoreUtxoId> for UtxoId {
    fn from(value: &FuelCoreUtxoId) -> Self {
        Self {
            tx_id: value.tx_id().into(),
            output_index: value.output_index(),
        }
    }
}
impl From<UtxoId> for HexData {
    fn from(value: UtxoId) -> Self {
        let mut bytes = Vec::with_capacity(34);
        bytes.extend_from_slice(value.tx_id.0.as_ref());
        bytes.extend_from_slice(&value.output_index.to_be_bytes());
        HexData(bytes.into())
    }
}

#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct PanicInstruction {
    pub reason: PanicReason,
    pub instruction: RawInstruction,
}
impl From<FuelCorePanicInstruction> for PanicInstruction {
    fn from(value: FuelCorePanicInstruction) -> Self {
        Self {
            reason: value.reason().to_owned(),
            instruction: value.instruction().to_owned(),
        }
    }
}

#[derive(
    Debug,
    Copy,
    Clone,
    PartialEq,
    Eq,
    Hash,
    Default,
    serde::Serialize,
    serde::Deserialize,
)]
#[repr(u64)]
pub enum ScriptExecutionResult {
    Success,
    Revert,
    Panic,
    // Generic failure case since any u64 is valid here
    GenericFailure(u64),
    #[default]
    Unknown,
}
impl From<FuelCoreScriptExecutionResult> for ScriptExecutionResult {
    fn from(value: FuelCoreScriptExecutionResult) -> Self {
        match value {
            FuelCoreScriptExecutionResult::Success => Self::Success,
            FuelCoreScriptExecutionResult::Revert => Self::Revert,
            FuelCoreScriptExecutionResult::Panic => Self::Panic,
            FuelCoreScriptExecutionResult::GenericFailure(value) => {
                Self::GenericFailure(value)
            }
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
