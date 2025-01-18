use fuel_streams_macros::subject::*;
use fuel_streams_types::*;
use serde::{Deserialize, Serialize};

use super::types::*;
use crate::blocks::types::*;

#[derive(Subject, Debug, Clone, Default, Serialize, Deserialize)]
#[subject(id = "utxos")]
#[subject(entity = "Utxo")]
#[subject(wildcard = "utxos.>")]
#[subject(
    format = "utxos.{block_height}.{tx_id}.{tx_index}.{input_index}.{utxo_type}.{utxo_id}"
)]
pub struct UtxosSubject {
    pub block_height: Option<BlockHeight>,
    pub tx_id: Option<TxId>,
    pub tx_index: Option<u32>,
    pub input_index: Option<u32>,
    pub utxo_type: Option<UtxoType>,
    pub utxo_id: Option<HexData>,
}
