use std::error::Error;

pub use fuel_core_types::{
    fuel_tx,
    fuel_types::{AssetId, ChainId},
    services::block_importer::ImportResult,
};

pub use crate::{blocks::types::*, nats::types::*, transactions::types::*};

// ------------------------------------------------------------------------
// General
// ------------------------------------------------------------------------

pub type BoxedResult<T> = Result<T, Box<dyn Error>>;

/// Macro to generate a wrapper type for different byte-based types (including Address type).
/// It provides the From trait implementation, Display formatting, and a zeroed method.
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
