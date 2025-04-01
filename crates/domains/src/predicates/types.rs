use std::str::FromStr;

use fuel_data_parser::DataEncoder;
use fuel_streams_types::primitives::*;
use serde::{Deserialize, Serialize};

use super::PredicateDbItem;
use crate::infra::{record::ToPacket, DbError};

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
    pub blob_id: Option<HexData>,
    pub predicate_address: Address,
    pub predicate_bytecode: HexData,
    pub tx_id: TxId,
    pub tx_index: u32,
    pub input_index: u32,
    pub asset_id: AssetId,
}

impl DataEncoder for Predicate {}
impl ToPacket for Predicate {}

impl Predicate {
    pub fn new(
        tx_id: &TxId,
        tx_index: u32,
        input_index: u32,
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

impl TryFrom<&PredicateDbItem> for Predicate {
    type Error = DbError;

    fn try_from(item: &PredicateDbItem) -> Result<Self, Self::Error> {
        let tx_id = TxId::from_str(&item.tx_id)
            .map_err(|e| DbError::Other(e.to_string()))?;
        let blob_id = item
            .blob_id
            .as_ref()
            .and_then(|b| HexData::from_str(&b).ok());
        let predicate_address = Address::from_str(&item.predicate_address)
            .map_err(|e| DbError::Other(e.to_string()))?;
        let predicate_bytecode = HexData::from_str(&item.bytecode)
            .map_err(|e| DbError::Other(e.to_string()))?;
        let asset_id = AssetId::from_str(&item.asset_id)
            .map_err(|e| DbError::Other(e.to_string()))?;
        let predicate = Predicate::new(
            &tx_id,
            item.tx_index as u32,
            item.input_index as u32,
            blob_id,
            &predicate_address,
            &predicate_bytecode,
            &asset_id,
        );
        Ok(predicate)
    }
}
