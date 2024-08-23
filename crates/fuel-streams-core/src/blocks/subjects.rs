use fuel_streams_macros::subject::{IntoSubject, Subject};

use crate::types::*;

#[derive(Subject, Debug, Clone, Default)]
#[subject_wildcard = "blocks.>"]
#[subject_format = "blocks.{producer}.{height}"]
pub struct BlocksSubject {
    pub producer: Option<Address>,
    pub height: Option<BlockHeight>,
}

impl From<&Block> for BlocksSubject {
    fn from(block: &Block) -> Self {
        let block_height = *block.header().height();
        BlocksSubject::new().with_height(Some(BlockHeight::from(block_height)))
    }
}

#[cfg(test)]
mod test {
    use fuel_streams_macros::subject::IntoSubject;
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn block_subjects_all() {
        assert_eq!(BlocksSubject::WILDCARD, "blocks.>")
    }

    #[test]
    fn block_subjects_parse() {
        let subject = BlocksSubject {
            producer: Some(Address::zeroed()),
            height: Some(23.into()),
        };
        assert_eq!(subject.parse(), "blocks.0x0000000000000000000000000000000000000000000000000000000000000000.23");
    }

    #[test]
    fn block_subjects_wildcard() {
        let wildcard = BlocksSubject::wildcard(None, Some(23.into()));
        assert_eq!(wildcard, "blocks.*.23")
    }

    #[test]
    fn block_subjects_builder() {
        let subject = BlocksSubject::new().with_height(Some(23.into()));
        assert_eq!(subject.parse(), "blocks.*.23")
    }

    #[test]
    fn block_subjects_from_block() {
        let mock_block = &MockBlock::build(1);
        let subject: BlocksSubject = mock_block.into();

        assert!(subject.producer.is_none());
        assert_eq!(subject.height.unwrap(), mock_block.clone().into());
    }
}
