use std::error::Error;

pub use crate::nats::types as nats;

// --------------------------------------------------------------------------------
// General
// --------------------------------------------------------------------------------

pub type BoxedResult<T> = Result<T, Box<dyn Error>>;
pub type BlockHeight = u32;
pub type Address = String;

// --------------------------------------------------------------------------------
// Identifier Kind
// --------------------------------------------------------------------------------

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

// --------------------------------------------------------------------------------
// Transaction
// --------------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub enum TransactionKind {
    Create,
    Mint,
    Script,
    Upgrade,
    Upload,
}

impl std::fmt::Display for TransactionKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let value: &'static str = match self {
            TransactionKind::Create => "create",
            TransactionKind::Mint => "mint",
            TransactionKind::Script => "script",
            TransactionKind::Upgrade => "upgrade",
            TransactionKind::Upload => "upload",
        };
        write!(f, "{value}")
    }
}

#[derive(Debug, Clone)]
pub enum TransactionStatus {
    Failed,
    Submitted,
    SqueezedOut,
    Success,
}

impl std::fmt::Display for TransactionStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let value: &'static str = match self {
            TransactionStatus::Failed => "failed",
            TransactionStatus::Submitted => "submitted",
            TransactionStatus::SqueezedOut => "squeezed_out",
            TransactionStatus::Success => "success",
        };
        write!(f, "{value}")
    }
}

pub use fuel_core_types::{blockchain::block::Block, fuel_tx::Transaction};
