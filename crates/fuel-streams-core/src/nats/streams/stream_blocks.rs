use std::fmt;

use super::{stream, subject::Subject};

pub mod subjects {
    use crate::nats::streams::subject;

    #[derive(Debug, Clone, Default)]
    pub struct Blocks {
        pub producer: Option<crate::types::Address>,
        pub height: Option<crate::types::BlockHeight>,
    }
    impl subject::Subject for Blocks {
        const WILDCARD: &'static str = "blocks.*.*";

        fn parse(&self) -> String {
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

impl stream::StreamSubjects for BlockSubjects {}
impl stream::StreamIdentifier for stream::Stream<BlockSubjects> {
    const STREAM: &'static str = "blocks";
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;
    use crate::types::BoxedResult;

    #[test]
    fn can_parse_subjecs() -> BoxedResult<()> {
        let subject = subjects::Blocks {
            producer: Some("0x000".to_string()),
            height: Some(100_u32),
        };
        let parsed = subject.parse().to_string();
        assert_eq!(parsed, "blocks.0x000.100");
        Ok(())
    }
}
