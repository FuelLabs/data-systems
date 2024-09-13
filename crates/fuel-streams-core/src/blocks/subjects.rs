use fuel_streams_macros::subject::{IntoSubject, Subject, SubjectBuildable};

use crate::types::*;

/// Represents a NATS subject for blocks in the Fuel network.
///
/// This subject format allows for efficient querying and filtering of blocks
/// based on their producer and height.
///
/// # Examples
///
/// Creating a subject for a specific block:
///
/// ```
/// # use fuel_streams_core::blocks::BlocksSubject;
/// # use fuel_streams_core::types::*;
/// # use fuel_streams_macros::subject::IntoSubject;
/// let subject = BlocksSubject {
///     producer: Some(Address::zeroed()),
///     height: Some(23.into()),
/// };
/// assert_eq!(subject.parse(), "blocks.0x0000000000000000000000000000000000000000000000000000000000000000.23");
/// ```
///
/// All blocks wildcard:
///
/// ```
/// # use fuel_streams_core::blocks::BlocksSubject;
/// assert_eq!(BlocksSubject::WILDCARD, "blocks.>");
/// ```
///
/// Creating a subject query using the `wildcard` method for flexible parameter-based filtering
///
/// ```
/// # use fuel_streams_core::blocks::BlocksSubject;
/// # use fuel_streams_core::types::*;
/// let wildcard = BlocksSubject::wildcard(None, Some(23.into()));
/// assert_eq!(wildcard, "blocks.*.23");
/// ```
///
/// Using the builder pattern for flexible subject construction:
/// This approach allows for step-by-step creation of a `BlocksSubject`,
///
/// ```
/// # use fuel_streams_core::blocks::BlocksSubject;
/// # use fuel_streams_core::types::*;
/// # use fuel_streams_macros::subject::*;
/// let subject = BlocksSubject::new()
///     .with_producer(Some(Address::zeroed()))
///     .with_height(Some(23.into()));
/// assert_eq!(subject.parse(), "blocks.0x0000000000000000000000000000000000000000000000000000000000000000.23");
/// ```
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
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn block_subjects_from_block() {
        let mock_block = &MockBlock::build(1);
        let subject: BlocksSubject = mock_block.into();

        assert!(subject.producer.is_none());
        assert_eq!(subject.height.unwrap(), mock_block.clone().into());
    }
}
