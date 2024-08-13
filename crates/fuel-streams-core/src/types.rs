use std::error::Error;

pub use crate::{blocks::types::*, nats::types::*, transactions::types::*};

// ------------------------------------------------------------------------
// General
// ------------------------------------------------------------------------

pub type BoxedResult<T> = Result<T, Box<dyn Error>>;
pub type Address = String;

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
