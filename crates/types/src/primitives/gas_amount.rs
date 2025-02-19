use crate::declare_integer_wrapper;

#[derive(thiserror::Error, Debug, Clone, PartialEq)]
pub enum GasAmountError {
    #[error("Failed to parse to gas_amount: {0}")]
    InvalidFormat(String),
}

declare_integer_wrapper!(GasAmount, u64, GasAmountError);
