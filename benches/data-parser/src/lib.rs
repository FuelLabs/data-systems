use fuel_core_types::{
    blockchain::{
        block::{Block, BlockV1},
        header::{ApplicationHeader, ConsensusHeader},
        primitives::DaBlockHeight,
    },
    fuel_tx::{Bytes32, Transaction},
    fuel_types::BlockHeight,
    tai64::Tai64,
};
use rand::Rng;

pub fn generate_test_block() -> Block<Transaction> {
    let mut rng = rand::thread_rng();
    let block_height: u32 = rng.gen_range(1..100);
    let block_txs: u32 = rng.gen_range(1..100);
    let previous_root: [u8; 32] = rng.gen();
    let tx_root: [u8; 32] = rng.gen();
    let mut block: Block<Transaction> = Block::V1(BlockV1::default());
    let txs = (0..block_txs)
        .map(|_| Transaction::default_test_tx())
        .collect::<Vec<_>>();
    *block.transactions_mut() = txs;
    block
        .header_mut()
        .set_application_header(ApplicationHeader::default());
    block
        .header_mut()
        .set_block_height(BlockHeight::new(block_height));
    block
        .header_mut()
        .set_consensus_header(ConsensusHeader::default());
    block
        .header_mut()
        .set_da_height(DaBlockHeight::from(block_height as u64));
    block
        .header_mut()
        .set_previous_root(Bytes32::new(previous_root));
    block.header_mut().set_time(Tai64::now());
    block
        .header_mut()
        .set_transaction_root(Bytes32::new(tx_root));
    block
}

pub fn generate_test_tx() -> Transaction {
    Transaction::default_test_tx()
}
