use fuel_streams_domains::blocks::{Block, MockBlock};
use rand::Rng;

pub fn generate_test_block() -> Block {
    let mut rng = rand::thread_rng();
    let block_height: u32 = rng.gen_range(1..100000);
    MockBlock::build(block_height)
}
