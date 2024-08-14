use core::fmt;

// ------------------------------------------------------------------------
// Block
// ------------------------------------------------------------------------
pub use fuel_core_types::{
    blockchain::block::{Block, BlockV1},
    fuel_types,
};

#[derive(Debug, Clone)]
#[cfg(any(test, feature = "test-helpers"))]
pub struct MockBlock(pub Block);

#[cfg(any(test, feature = "test-helpers"))]
impl MockBlock {
    pub fn build(height: u32) -> Block {
        use crate::transactions::types::Transaction;
        let mut block: Block<Transaction> = Block::V1(BlockV1::default());
        block
            .header_mut()
            .set_block_height(fuel_types::BlockHeight::new(height));

        let txs = (0..50)
            .map(|_| Transaction::default_test_tx())
            .collect::<Vec<_>>();
        *block.transactions_mut() = txs;
        block
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct BlockHeight(String);

impl From<Block> for BlockHeight {
    fn from(value: Block) -> Self {
        let height = *value.header().consensus().height;
        BlockHeight(height.to_string())
    }
}

impl From<fuel_types::BlockHeight> for BlockHeight {
    fn from(value: fuel_types::BlockHeight) -> Self {
        let height = *value;
        BlockHeight(height.to_string())
    }
}

impl From<u32> for BlockHeight {
    fn from(value: u32) -> Self {
        BlockHeight::from(fuel_types::BlockHeight::from(value))
    }
}

impl fmt::Display for BlockHeight {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}
