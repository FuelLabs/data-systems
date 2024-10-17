// ------------------------------------------------------------------------
// Transaction
// ------------------------------------------------------------------------
pub use fuel_core_types::{
    fuel_tx::Transaction,
    services::txpool::TransactionStatus as FuelCoreTransactionStatus,
};

#[cfg(any(test, feature = "test-helpers"))]
use crate::types::*;

#[derive(Debug, Clone)]
#[cfg(any(test, feature = "test-helpers"))]
pub struct MockTransaction(pub Block);

#[cfg(any(test, feature = "test-helpers"))]
impl MockTransaction {
    pub fn build() -> Transaction {
        Transaction::default_test_tx()
    }
}

#[derive(Debug, Clone)]
pub enum TransactionKind {
    Create,
    Mint,
    Script,
    Upgrade,
    Upload,
    Blob,
}

impl std::fmt::Display for TransactionKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let value: &'static str = match self {
            TransactionKind::Create => "create",
            TransactionKind::Mint => "mint",
            TransactionKind::Script => "script",
            TransactionKind::Upgrade => "upgrade",
            TransactionKind::Upload => "upload",
            TransactionKind::Blob => "blob",
        };
        write!(f, "{value}")
    }
}

impl From<Transaction> for TransactionKind {
    fn from(value: Transaction) -> Self {
        match value {
            Transaction::Script(_) => TransactionKind::Script,
            Transaction::Create(_) => TransactionKind::Create,
            Transaction::Mint(_) => TransactionKind::Mint,
            Transaction::Upgrade(_) => TransactionKind::Upgrade,
            Transaction::Upload(_) => TransactionKind::Upload,
            Transaction::Blob(_) => TransactionKind::Blob,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub enum TransactionStatus {
    Failed,
    Submitted,
    SqueezedOut,
    Success,
    #[default]
    None,
}

impl std::fmt::Display for TransactionStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let value: &'static str = match self {
            TransactionStatus::Failed => "failed",
            TransactionStatus::Submitted => "submitted",
            TransactionStatus::SqueezedOut => "squeezed_out",
            TransactionStatus::Success => "success",
            TransactionStatus::None => "none",
        };
        write!(f, "{value}")
    }
}

impl From<FuelCoreTransactionStatus> for TransactionStatus {
    fn from(value: FuelCoreTransactionStatus) -> Self {
        match value {
            FuelCoreTransactionStatus::Failed { .. } => {
                TransactionStatus::Failed
            }
            FuelCoreTransactionStatus::Submitted { .. } => {
                TransactionStatus::Submitted
            }
            FuelCoreTransactionStatus::SqueezedOut { .. } => {
                TransactionStatus::SqueezedOut
            }
            FuelCoreTransactionStatus::Success { .. } => {
                TransactionStatus::Success
            }
        }
    }
}
