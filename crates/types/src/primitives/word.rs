use crate::{declare_integer_wrapper, impl_utoipa_for_integer_wrapper};

#[derive(thiserror::Error, Debug)]
pub enum WordError {
    #[error("Invalid format: {0}")]
    InvalidFormat(String),
}

declare_integer_wrapper!(Word, u64, WordError);

impl_utoipa_for_integer_wrapper!(
    Word,
    "A word in the blockchain",
    0,
    u64::MAX as usize
);
