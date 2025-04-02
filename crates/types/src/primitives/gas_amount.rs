use crate::{declare_integer_wrapper, impl_utoipa_for_integer_wrapper};

declare_integer_wrapper!(GasAmount, u64);

impl_utoipa_for_integer_wrapper!(
    GasAmount,
    "Gas Amount in the blockchain",
    0,
    u64::MAX as usize
);
