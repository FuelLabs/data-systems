use fuel_data_parser::DataEncoder;
use fuel_streams_types::primitives::*;
use serde::{Deserialize, Serialize};

use crate::infra::record::ToPacket;

#[derive(
    Debug,
    Clone,
    Default,
    Serialize,
    Deserialize,
    PartialEq,
    Eq,
    utoipa::ToSchema,
)]
#[serde(rename_all = "camelCase")]
pub struct Predicate {
    pub tx_id: TxId,
    pub blob_id: Option<HexData>,
    pub predicate_address: Address,
    pub predicate_bytecode: HexData,
}

impl DataEncoder for Predicate {}
impl ToPacket for Predicate {}

impl Predicate {
    pub fn new(
        tx_id: &TxId,
        blob_id: Option<&HexData>,
        predicate_address: &Address,
        predicate_bytecode: &HexData,
    ) -> Self {
        Self {
            tx_id: tx_id.clone(),
            blob_id: blob_id.map(|b| b.to_owned()),
            predicate_address: predicate_address.clone(),
            predicate_bytecode: predicate_bytecode.clone(),
        }
    }
}
