use std::error::Error;

pub use fuel_core_types::{
    fuel_tx,
    fuel_types,
    fuel_types::ChainId,
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
/// ```
/// generate_byte_type_wrapper!(WrapperType, InnerType);
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

/// Macro to generate a wrapper type for different byte-based types.
///
/// This macro creates a new struct that wraps the specified inner type,
/// typically used for various byte-based identifiers in the Fuel ecosystem.
/// It automatically implements:
///
/// - `From<inner_type>` for easy conversion
/// - `Display` for formatting (prefixes with "0x")
/// - `PartialEq` for comparison
/// - A `zeroed()` method to create an instance filled with zeros
///
/// # Arguments
///
/// * `$wrapper_type` - The name of the new wrapper type to be created
/// * `$inner_type` - The inner type being wrapped (e.g., `fuel_tx::Address`)
///
/// # Example
///
/// ```
/// generate_byte_type_wrapper!(MyCustomId, [u8; 32]);
/// ```
///
/// This would generate a `MyCustomId` struct wrapping a `[u8; 32]` array.
macro_rules! impl_from_for_bytes32 {
    ($from_type:ty) => {
        impl From<$from_type> for Bytes32 {
            fn from(value: $from_type) -> Self {
                Bytes32(fuel_core_types::fuel_tx::Bytes32::from(*value))
            }
        }
    };
}

impl_from_for_bytes32!(fuel_tx::ContractId);
impl_from_for_bytes32!(fuel_types::AssetId);
impl_from_for_bytes32!(fuel_tx::Address);

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
