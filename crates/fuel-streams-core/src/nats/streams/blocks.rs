use super::{
    stream::Streamable,
    subject::{self, Subject},
};
use crate::types::Block;

#[derive(Debug, Clone, Default)]
pub struct BlocksSubject {
    pub producer: Option<crate::types::Address>,
    pub height: Option<crate::types::BlockHeight>,
}

impl Subject for BlocksSubject {
    const WILDCARD: &'static str = "blocks.*.*";

    fn parse(&self) -> String {
        let producer = subject::parse_param(&self.producer);
        let height = subject::parse_param(&self.height);
        format!("blocks.{producer}.{height}")
    }
}

impl Streamable for Block {
    const STREAM: &'static str = "blocks";
    const SUBJECTS_WILDCARDS: &'static [&'static str] =
        &[BlocksSubject::WILDCARD];
}
