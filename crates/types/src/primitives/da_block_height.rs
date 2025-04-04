use fuel_core_types::blockchain::primitives::DaBlockHeight as FuelCoreDaBlockHeight;

use crate::{
    declare_integer_wrapper,
    impl_avro_schema_for_wrapped_int,
    impl_utoipa_for_integer_wrapper,
};

declare_integer_wrapper!(DaBlockHeight, u64);
impl_avro_schema_for_wrapped_int!(DaBlockHeight, u64);

impl_utoipa_for_integer_wrapper!(
    DaBlockHeight,
    "Da Block height in the blockchain",
    0,
    u64::MAX as usize
);

impl From<FuelCoreDaBlockHeight> for DaBlockHeight {
    fn from(value: FuelCoreDaBlockHeight) -> Self {
        value.0.into()
    }
}
