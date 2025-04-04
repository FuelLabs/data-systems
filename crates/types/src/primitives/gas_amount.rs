use crate::{
    declare_integer_wrapper,
    impl_avro_schema_for_wrapped_int,
    impl_utoipa_for_integer_wrapper,
};

declare_integer_wrapper!(GasAmount, u64);
impl_avro_schema_for_wrapped_int!(GasAmount, u64);

impl_utoipa_for_integer_wrapper!(
    GasAmount,
    "Gas Amount in the blockchain",
    0,
    u64::MAX as usize
);
