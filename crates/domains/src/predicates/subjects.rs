use fuel_streams_subject::subject::*;
use fuel_streams_types::*;
use serde::{Deserialize, Serialize};

use super::PredicatesQuery;

#[derive(Subject, Debug, Clone, Default, Serialize, Deserialize)]
#[subject(id = "predicates")]
#[subject(entity = "Predicate")]
#[subject(query_all = "predicates.>")]
#[subject(
    format = "predicates.{block_height}.{tx_id}.{tx_index}.{input_index}.{blob_id}.{predicate_address}.{asset}"
)]
pub struct PredicatesSubject {
    #[subject(
        description = "The height of the block containing this predicate"
    )]
    pub block_height: Option<BlockHeight>,
    #[subject(
        description = "The ID of the transaction containing this predicate (32 byte string prefixed by 0x)"
    )]
    pub tx_id: Option<TxId>,
    #[subject(description = "The index of the transaction within the block")]
    pub tx_index: Option<i32>,
    #[subject(
        description = "The index of this input within the transaction that had this predicate"
    )]
    pub input_index: Option<i32>,
    #[subject(
        description = "The ID of the blob containing the predicate bytecode"
    )]
    pub blob_id: Option<HexData>,
    #[subject(
        description = "The address of the predicate (32 byte string prefixed by 0x)"
    )]
    pub predicate_address: Option<Address>,
    #[subject(
        sql_column = "asset_id",
        description = "The asset ID of the coin (32 byte string prefixed by 0x)"
    )]
    pub asset: Option<AssetId>,
}

impl PredicatesSubject {
    pub fn to_sql_where(&self) -> Option<String> {
        let mut conditions = Vec::new();

        if let Some(block_height) = self.block_height {
            conditions.push(format!("pt.block_height = '{}'", block_height));
        }
        if let Some(tx_id) = &self.tx_id {
            conditions.push(format!("pt.tx_id = '{}'", tx_id));
        }
        if let Some(tx_index) = self.tx_index {
            conditions.push(format!("pt.tx_index = '{}'", tx_index));
        }
        if let Some(input_index) = self.input_index {
            conditions.push(format!("pt.input_index = '{}'", input_index));
        }
        if let Some(blob_id) = &self.blob_id {
            conditions.push(format!("p.blob_id = '{}'", blob_id));
        }
        if let Some(predicate_address) = &self.predicate_address {
            conditions
                .push(format!("p.predicate_address = '{}'", predicate_address));
        }
        if let Some(asset) = &self.asset {
            conditions.push(format!("pt.asset_id = '{}'", asset));
        }

        if conditions.is_empty() {
            None
        } else {
            Some(conditions.join(" AND "))
        }
    }
}

impl From<PredicatesSubject> for PredicatesQuery {
    fn from(subject: PredicatesSubject) -> Self {
        Self {
            block_height: subject.block_height,
            tx_id: subject.tx_id.clone(),
            tx_index: subject.tx_index,
            input_index: subject.input_index,
            blob_id: subject.blob_id.clone(),
            predicate_address: subject.predicate_address.clone(),
            asset: subject.asset.clone(),
            ..Default::default()
        }
    }
}
