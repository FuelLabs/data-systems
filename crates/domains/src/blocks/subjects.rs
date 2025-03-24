#![allow(unused_variables)]
#![allow(clippy::too_many_arguments)]

use fuel_streams_subject::subject::*;
use fuel_streams_types::*;
use serde::{Deserialize, Serialize};

use super::types::*;

#[derive(
    Subject, Debug, Clone, Default, Serialize, Deserialize, Eq, PartialEq,
)]
#[subject(id = "blocks")]
#[subject(entity = "Block")]
#[subject(query_all = "blocks.>")]
#[subject(format = "blocks.{producer}.{da_height}.{height}")]
pub struct BlocksSubject {
    #[subject(
        sql_column = "block_height",
        description = "The height of the block as unsigned 64 bit integer"
    )]
    pub height: Option<BlockHeight>,

    #[subject(
        sql_column = "block_da_height",
        description = "The height of the Data Availability layer block as unsigned 64 bit integer"
    )]
    pub da_height: Option<DaBlockHeight>,

    #[subject(sql_column = "version", description = "The block version")]
    pub version: Option<BlockVersion>,

    #[subject(
        sql_column = "producer_address",
        description = "The address of the producer that created the block"
    )]
    pub producer: Option<Address>,
}

impl From<&Block> for BlocksSubject {
    fn from(block: &Block) -> Self {
        let (
            consensus_type,
            consensus_chain_config_hash,
            consensus_coins_root,
            consensus_contracts_root,
            consensus_messages_root,
            consensus_transactions_root,
            consensus_signature,
        ) = block.consensus.normalize_all();

        BlocksSubject {
            height: Some(block.height.to_owned()),
            da_height: Some(block.header.da_height.to_owned()),
            producer: Some(block.producer.to_owned()),
            version: Some(block.version.to_owned()),
        }
    }
}
