use fuel_streams_subject::subject::*;
use fuel_streams_types::*;
use serde::{Deserialize, Serialize};

use super::{types::*, BlocksQuery};

#[derive(
    Subject, Debug, Clone, Default, Serialize, Deserialize, Eq, PartialEq,
)]
#[subject(id = "blocks")]
#[subject(entity = "Block")]
#[subject(query_all = "blocks.>")]
#[subject(format = "blocks.{producer}.{da_height}.{height}")]
pub struct BlocksSubject {
    #[subject(
        sql_column = "producer_address",
        description = "The address of the producer that created the block"
    )]
    pub producer: Option<Address>,
    #[subject(
        sql_column = "block_da_height",
        description = "The height of the DA block as unsigned 64 bit integer"
    )]
    pub da_height: Option<DaBlockHeight>,
    #[subject(
        sql_column = "block_height",
        description = "The height of the block as unsigned 64 bit integer"
    )]
    pub height: Option<BlockHeight>,
}

impl From<&Block> for BlocksSubject {
    fn from(block: &Block) -> Self {
        BlocksSubject {
            producer: Some(block.producer.to_owned()),
            da_height: Some(block.header.da_height.to_owned()),
            height: Some(block.height.to_owned()),
        }
    }
}

impl From<BlocksSubject> for BlocksQuery {
    fn from(subject: BlocksSubject) -> Self {
        Self {
            producer: subject.producer.clone(),
            height: subject.height,
            ..Default::default()
        }
    }
}
