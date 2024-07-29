use std::fmt;

use super::{stream, subject::Subject};

pub(super) mod subjects {
    use crate::nats::streams::subject;

    #[derive(Debug, Clone, Default)]
    pub struct Blocks {
        producer: Option<crate::types::Address>,
        height: Option<crate::types::BlockHeight>,
    }
    impl subject::Subject for Blocks {
        const WILDCARD: &'static str = "blocks.*.*";

        fn parse(&self) -> impl ToString {
            let producer = subject::parse_param(&self.producer);
            let height = subject::parse_param(&self.height);
            format!("blocks.{producer}.{height}")
        }
    }
}

#[derive(Debug, Clone, strum::EnumIter)]
pub enum BlockSubjects {
    Blocks(subjects::Blocks),
}

impl fmt::Display for BlockSubjects {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let value = match self {
            BlockSubjects::Blocks(s) => s.wildcard(),
        };
        write!(f, "{}", value)
    }
}

impl stream::StreamSubjectsEnum for BlockSubjects {}
impl stream::StreamIdentifier for stream::Stream<BlockSubjects> {
    const STREAM: &'static str = "blocks";
}
