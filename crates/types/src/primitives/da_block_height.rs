use fuel_core_types::blockchain::primitives::DaBlockHeight as FuelCoreDaBlockHeight;

use crate::{declare_integer_wrapper, impl_utoipa_for_integer_wrapper};

#[derive(thiserror::Error, Debug)]
pub enum DaBlockHeightError {
    #[error("Failed to parse to block_height: {0}")]
    InvalidFormat(String),
}

declare_integer_wrapper!(DaBlockHeight, u64, DaBlockHeightError);

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
