use std::{collections::HashMap, sync::Arc};

use fuel_core::database::database_description::DatabaseHeight;
use fuel_core_types::blockchain::consensus::Sealed;
use fuel_streams_core::prelude::*;
use itertools::Itertools;
use parking_lot::Mutex;

/// Manages a collection of unpublished blocks for sequential publishing
///
/// This was introduced to allow the publisher sequentially publish each block even though
/// it will receive them out of order from two different streams, the block importer stream and
/// the old blocks stream
///
/// # Key Features
/// - Thread-safe block storage using `Arc<Mutex<>>`
/// - Sequential block publishing
/// - Ability to handle blocks received out of order
///
/// ```no_compile
/// let unpublished_blocks = UnpublishedBlocks::new(10);
/// let block11 = create_mock_block(11);
/// let block12 = create_mock_block(12);
/// unpublished_blocks.add_block(block11);
/// unpublished_blocks.add_block(block12);
/// let next_blocks = unpublished_blocks.get_next_blocks_to_publish();
/// ```
#[derive(Debug, Default, Clone)]
pub struct UnpublishedBlocks {
    inner: Arc<Mutex<UnpublishedBlocksInner>>,
}

#[derive(Debug, Default, Clone)]
struct UnpublishedBlocksInner {
    blocks: HashMap<u64, Sealed<FuelCoreBlock<FuelCoreTransaction>>>,
    next_height_to_publish: u64,
}

impl UnpublishedBlocks {
    pub fn new(last_published_block_height: u64) -> Self {
        Self {
            inner: Arc::new(Mutex::new(UnpublishedBlocksInner {
                blocks: HashMap::new(),
                next_height_to_publish: last_published_block_height,
            })),
        }
    }

    pub fn add_block(&self, block: Sealed<FuelCoreBlock<FuelCoreTransaction>>) {
        let height = block.entity.header().consensus().height.as_u64();
        self.inner.lock().blocks.insert(height, block);
    }

    pub fn get_next_blocks_to_publish(
        &self,
    ) -> Vec<Sealed<FuelCoreBlock<FuelCoreTransaction>>> {
        let mut next_blocks_to_publish = vec![];
        let mut unpublished_blocks = self.inner.lock();

        let unpublished_heights: Vec<_> =
            unpublished_blocks.blocks.keys().sorted().cloned().collect();

        for unpublished_height in unpublished_heights {
            if unpublished_height == unpublished_blocks.next_height_to_publish {
                let block = unpublished_blocks
                    .blocks
                    .remove(&unpublished_height)
                    .unwrap();
                next_blocks_to_publish.push(block);
                unpublished_blocks.next_height_to_publish += 1;
            } else {
                break;
            }
        }

        next_blocks_to_publish
    }
}
#[cfg(test)]
mod tests {
    use fuel_core_types::blockchain::SealedBlock;

    use super::*;

    #[test]
    fn test_new_unpublished_blocks() {
        let last_published_block_height = 10;
        let unpublished_blocks =
            UnpublishedBlocks::new(last_published_block_height);

        let inner = unpublished_blocks.inner.lock();
        assert!(inner.blocks.is_empty());
        assert_eq!(inner.next_height_to_publish, last_published_block_height);
    }

    #[test]
    fn test_add_block() {
        let unpublished_blocks = UnpublishedBlocks::new(11);

        let block1 = create_mock_block(11);
        let block2 = create_mock_block(12);

        unpublished_blocks.add_block(block1.clone());
        unpublished_blocks.add_block(block2.clone());

        let inner = unpublished_blocks.inner.lock();
        assert_eq!(inner.blocks.len(), 2);
        assert!(inner.blocks.contains_key(&11));
        assert!(inner.blocks.contains_key(&12));
    }

    #[test]
    fn test_add_block_overrides_existing_block() {
        let unpublished_blocks = UnpublishedBlocks::new(11);

        let block1 = create_mock_block(11);
        let block2 = create_mock_block(11); // Same height, different block

        unpublished_blocks.add_block(block1.clone());
        unpublished_blocks.add_block(block2.clone());

        let inner = unpublished_blocks.inner.lock();
        assert_eq!(inner.blocks.len(), 1); // Replaced the earlier block
        assert!(inner.blocks.contains_key(&11));
    }

    #[test]
    fn test_get_next_blocks_to_publish_sequential() {
        let unpublished_blocks = UnpublishedBlocks::new(11);

        unpublished_blocks.add_block(create_mock_block(11));
        unpublished_blocks.add_block(create_mock_block(12));
        unpublished_blocks.add_block(create_mock_block(13));

        let next_blocks = unpublished_blocks.get_next_blocks_to_publish();
        assert_eq!(next_blocks.len(), 3);
        assert_eq!(
            next_blocks[0].entity.header().consensus().height.as_u64(),
            11
        );
        assert_eq!(
            next_blocks[1].entity.header().consensus().height.as_u64(),
            12
        );
        assert_eq!(
            next_blocks[2].entity.header().consensus().height.as_u64(),
            13
        );
    }

    #[test]
    fn test_get_next_blocks_to_publish_non_sequential() {
        let unpublished_blocks = UnpublishedBlocks::new(11);

        unpublished_blocks.add_block(create_mock_block(12));
        unpublished_blocks.add_block(create_mock_block(11));
        unpublished_blocks.add_block(create_mock_block(13));
        unpublished_blocks.add_block(create_mock_block(14));
        unpublished_blocks.add_block(create_mock_block(17));
        unpublished_blocks.add_block(create_mock_block(19));
        unpublished_blocks.add_block(create_mock_block(100));

        let next_blocks = unpublished_blocks.get_next_blocks_to_publish();
        assert_eq!(next_blocks.len(), 4);
        assert_eq!(
            next_blocks[0].entity.header().consensus().height.as_u64(),
            11
        );
        assert_eq!(
            next_blocks[1].entity.header().consensus().height.as_u64(),
            12
        );
        assert_eq!(
            next_blocks[2].entity.header().consensus().height.as_u64(),
            13
        );
        assert_eq!(
            next_blocks[3].entity.header().consensus().height.as_u64(),
            14
        );

        assert!(unpublished_blocks.get_next_blocks_to_publish().is_empty());
    }

    #[test]
    fn test_get_next_blocks_to_returns_next_blocks_and_leaves_out_the_rest() {
        let unpublished_blocks = UnpublishedBlocks::new(1);

        unpublished_blocks.add_block(create_mock_block(1));
        unpublished_blocks.add_block(create_mock_block(2));
        unpublished_blocks.add_block(create_mock_block(11));

        let next_blocks = unpublished_blocks.get_next_blocks_to_publish();
        assert_eq!(next_blocks.len(), 2);

        let returned_heights: Vec<u64> = next_blocks
            .iter()
            .map(|block| block.entity.header().consensus().height.as_u64())
            .collect();

        assert_eq!(returned_heights, vec![1, 2]);

        let inner = unpublished_blocks.inner.lock();
        assert_eq!(inner.next_height_to_publish, 3);
        assert_eq!(inner.blocks.len(), 1);
        assert!(inner.blocks.contains_key(&11));
    }

    #[test]
    fn test_next_height_increments_correctly() {
        let unpublished_blocks = UnpublishedBlocks::new(11);

        unpublished_blocks.add_block(create_mock_block(11));
        unpublished_blocks.add_block(create_mock_block(12));

        let next_blocks_to_publish =
            unpublished_blocks.get_next_blocks_to_publish();

        assert_eq!(next_blocks_to_publish.len(), 2);

        let inner = unpublished_blocks.inner.lock();
        assert_eq!(inner.next_height_to_publish, 13);
        assert!(inner.blocks.is_empty());
    }

    #[test]
    fn test_large_block_gap() {
        let unpublished_blocks = UnpublishedBlocks::new(1);

        // Add blocks with large gaps
        unpublished_blocks.add_block(create_mock_block(1));
        unpublished_blocks.add_block(create_mock_block(10));
        unpublished_blocks.add_block(create_mock_block(11));

        let next_blocks = unpublished_blocks.get_next_blocks_to_publish();
        assert_eq!(next_blocks.len(), 1);
        assert_eq!(
            next_blocks[0].entity.header().consensus().height.as_u64(),
            1
        );

        let inner = unpublished_blocks.inner.lock();
        assert_eq!(inner.next_height_to_publish, 2);
        assert_eq!(inner.blocks.len(), 2); // 10 and 11 remain
    }

    fn create_mock_block(
        height: u64,
    ) -> Sealed<FuelCoreBlock<FuelCoreTransaction>> {
        let mut block = FuelCoreBlock::default();

        block.header_mut().consensus_mut().height =
            FuelCoreBlockHeight::new(height as u32);

        SealedBlock {
            entity: block,
            ..Default::default()
        }
    }
}
