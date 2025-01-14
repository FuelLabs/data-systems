use fuel_streams_macros::subject::*;
use fuel_streams_types::*;
use serde::{Deserialize, Serialize};

use super::types::*;

#[derive(Subject, Debug, Clone, Default, Serialize, Deserialize)]
#[subject_id = "blocks"]
#[subject_wildcard = "blocks.>"]
#[subject_format = "blocks.{producer}.{block_height}"]
pub struct BlocksSubject {
    pub producer: Option<Address>,
    pub block_height: Option<BlockHeight>,
}

impl From<&Block> for BlocksSubject {
    fn from(block: &Block) -> Self {
        BlocksSubject::new().with_block_height(Some(block.height.clone()))
    }
}
