use fuel_streams_subject::subject::*;
use fuel_streams_types::*;
use serde::{Deserialize, Serialize};

#[derive(Subject, Debug, Clone, Default, Serialize, Deserialize)]
#[subject(id = "predicates")]
#[subject(entity = "Predicate")]
#[subject(query_all = "predicates.>")]
#[subject(
    format = "predicates.{block_height}.{tx_id}.{tx_index}.{input_index}.{blob_id}.{predicate_address}"
)]
pub struct PredicatesSubject {
    #[subject(description = "The height of the block containing this UTXO")]
    pub block_height: Option<BlockHeight>,
    #[subject(
        description = "The ID of the transaction containing this predicate (32 byte string prefixed by 0x)"
    )]
    pub tx_id: Option<TxId>,
    #[subject(description = "The index of the transaction within the block")]
    pub tx_index: Option<u32>,
    #[subject(
        description = "The index of this input within the transaction that had this predicate"
    )]
    pub input_index: Option<u32>,
    #[subject(
        description = "The ID of the blob containing the predicate bytecode"
    )]
    pub blob_id: Option<HexData>,
    #[subject(
        description = "The address of the predicate (32 byte string prefixed by 0x)"
    )]
    pub predicate_address: Option<Address>,
}
