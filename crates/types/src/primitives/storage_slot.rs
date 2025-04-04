use serde::{Deserialize, Serialize};

use super::Bytes32;
use crate::fuel_core::*;

#[derive(
    Debug, Default, Clone, PartialEq, Serialize, Deserialize, utoipa::ToSchema,
)]
pub struct StorageSlot {
    pub key: Bytes32,
    pub value: Bytes32,
}

impl From<FuelCoreStorageSlot> for StorageSlot {
    fn from(slot: FuelCoreStorageSlot) -> Self {
        Self::from(&slot)
    }
}

impl From<&FuelCoreStorageSlot> for StorageSlot {
    fn from(slot: &FuelCoreStorageSlot) -> Self {
        Self {
            key: slot.key().into(),
            value: slot.value().into(),
        }
    }
}
