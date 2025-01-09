use fuel_streams_macros::subject::*;
use fuel_streams_types::*;

use super::types::*;

#[derive(Subject, Debug, Clone, Default)]
#[subject_wildcard = "blocks.>"]
#[subject_format = "blocks.{producer}.{height}"]
pub struct BlocksSubject {
    pub producer: Option<Address>,
    pub height: Option<BlockHeight>,
}

impl From<&Block> for BlocksSubject {
    fn from(block: &Block) -> Self {
        BlocksSubject::new().with_height(Some(block.height.clone()))
    }
}
