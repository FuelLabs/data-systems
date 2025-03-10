use crate::{declare_integer_wrapper, impl_utoipa_for_integer_wrapper};

#[derive(thiserror::Error, Debug, Clone, PartialEq)]
pub enum AmountError {
    #[error("Failed to parse to amount: {0}")]
    InvalidFormat(String),
}

declare_integer_wrapper!(Amount, u64, AmountError);

impl_utoipa_for_integer_wrapper!(
    Amount,
    "Amount in the blockchain",
    0,
    u64::MAX as usize
);
