use std::error::Error;

// --------------------------------------------------------------------------------
// General
// --------------------------------------------------------------------------------

pub type BoxedResult<T> = Result<T, Box<dyn Error>>;

// --------------------------------------------------------------------------------
// Block
// --------------------------------------------------------------------------------

pub type BlockHeight = fuel_core_types::fuel_types::BlockHeight;

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
