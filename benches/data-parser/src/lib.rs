use fuel_streams_domains::blocks::{Block, MockBlock};
use rand::Rng;

pub fn generate_test_block() -> Block {
    let mut rng = rand::rng();
    let block_height = rng.random_range(1..100000);
    MockBlock::build(block_height.into())
}
