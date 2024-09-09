use std::error::Error;

pub use fuel_core_types::{
    fuel_tx,
    fuel_types::{self, ChainId},
    services::block_importer::ImportResult,
};

pub use crate::{
    blocks::types::*,
    inputs::types::*,
    nats::types::*,
    transactions::types::*,
};

// ------------------------------------------------------------------------
// General
// ------------------------------------------------------------------------

pub type BoxedResult<T> = Result<T, Box<dyn Error>>;

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
/// ```no_run
/// # use fuel_streams_core::generate_byte_type_wrapper;
/// generate_byte_type_wrapper!(AddressWrapped, fuel_core_types::fuel_tx::Address);
/// ```
///
/// Where `WrapperType` is the name of the new wrapper struct to be created,
/// and `InnerType` is the type being wrapped.
macro_rules! generate_byte_type_wrapper {
    ($wrapper_type:ident, $inner_type:ty) => {
        #[derive(Debug, Clone)]
        pub struct $wrapper_type($inner_type);

        impl From<$inner_type> for $wrapper_type {
            fn from(value: $inner_type) -> Self {
                $wrapper_type(value)
            }
        }

        impl From<[u8; 32]> for $wrapper_type {
            fn from(value: [u8; 32]) -> Self {
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

        impl PartialEq for $wrapper_type {
            fn eq(&self, other: &Self) -> bool {
                self.0 == other.0
            }
        }

        impl From<&str> for $wrapper_type {
            fn from(s: &str) -> Self {
                let s = s.strip_prefix("0x").unwrap_or(s);
                if s.len() != std::mem::size_of::<$inner_type>() * 2 {
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
                $wrapper_type(inner)
            }
        }

        impl $wrapper_type {
            pub fn zeroed() -> Self {
                $wrapper_type(<$inner_type>::zeroed())
            }
        }
    };
}

generate_byte_type_wrapper!(Address, fuel_tx::Address);
generate_byte_type_wrapper!(Bytes32, fuel_tx::Bytes32);
generate_byte_type_wrapper!(ContractId, fuel_tx::ContractId);
generate_byte_type_wrapper!(AssetId, fuel_types::AssetId);

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
    };
}


impl_from_bytes32!(fuel_tx::ContractId);
impl_from_bytes32!(fuel_types::AssetId);
impl_from_bytes32!(fuel_tx::Address);

// ------------------------------------------------------------------------
// Identifier
// ------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub enum IdentifierKind {
    Address,
    ContractID,
    AssetID,
}

impl std::fmt::Display for IdentifierKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let value: &'static str = match self {
            IdentifierKind::Address => "address",
            IdentifierKind::ContractID => "contract_id",
            IdentifierKind::AssetID => "asset_id",
        };
        write!(f, "{value}")
    }
}
