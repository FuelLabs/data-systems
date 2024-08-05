pub mod builder;
pub mod error;
pub mod parser;
pub mod types;

use fuel_core_types::{
    blockchain::block::{Block, BlockV1},
    fuel_tx::Transaction,
};

pub fn generate_test_block() -> Block<Transaction> {
    let mut block: Block<Transaction> = Block::V1(BlockV1::default());
    let txs = (0..50)
        .map(|_| Transaction::default_test_tx())
        .collect::<Vec<_>>();
    *block.transactions_mut() = txs;
    block
}

pub fn generate_test_tx() -> Transaction {
    Transaction::default_test_tx()
}
