use fuel_streams_macros::subject::*;
use fuel_streams_types::*;

use super::types::*;
use crate::blocks::types::*;

#[derive(Subject, Debug, Clone, Default)]
#[subject_wildcard = "utxos.>"]
#[subject_format = "utxos.{block_height}.{tx_id}.{tx_index}.{input_index}.{utxo_type}.{utxo_id}"]
pub struct UtxosSubject {
    pub block_height: Option<BlockHeight>,
    pub tx_id: Option<TxId>,
    pub tx_index: Option<u32>,
    pub input_index: Option<u32>,
    pub utxo_type: Option<UtxoType>,
    pub utxo_id: Option<HexData>,
}
