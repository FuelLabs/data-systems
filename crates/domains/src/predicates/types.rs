use fuel_data_parser::DataEncoder;
use fuel_streams_types::primitives::*;
use serde::{
    Deserialize,
    Serialize,
};

#[derive(
    Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq, utoipa::ToSchema,
)]
pub struct Predicate {
    pub blob_id: Option<HexData>,
    pub predicate_address: Address,
    pub predicate_bytecode: HexData,
    pub tx_id: TxId,
    pub tx_index: i32,
    pub input_index: i32,
    pub asset_id: AssetId,
}

impl DataEncoder for Predicate {}

impl Predicate {
    pub fn new(
        tx_id: &TxId,
        tx_index: i32,
        input_index: i32,
        blob_id: Option<HexData>,
        predicate_address: &Address,
        predicate_bytecode: &HexData,
        asset_id: &AssetId,
    ) -> Self {
        Self {
            blob_id,
            tx_id: tx_id.clone(),
            tx_index,
            input_index,
            predicate_address: predicate_address.clone(),
            predicate_bytecode: predicate_bytecode.clone(),
            asset_id: asset_id.to_owned(),
        }
    }
}
