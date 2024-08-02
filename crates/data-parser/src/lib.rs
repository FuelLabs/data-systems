pub mod builder;
pub mod error;
pub mod parser;
pub mod types;

use async_compression::Level;
use fuel_core_types::{
    blockchain::block::{Block, BlockV1},
    fuel_tx::{Mint, Transaction},
    fuel_types::{AssetId, ChainId},
};

use crate::{
    builder::DataParserBuilder,
    types::{CompressionType, SerializationType},
};

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, Eq, PartialEq)]
pub struct MyTestData {
    ids: Vec<String>,
    version: u64,
    receipts: Vec<String>,
    assets: Vec<AssetId>,
    chain_id: ChainId,
}

pub fn generate_test_block() -> Block<Mint> {
    let mut block: Block<Mint> = Block::V1(BlockV1::default());
    let txs = (0..100).map(|_| generate_test_tx()).collect::<Vec<_>>();
    *block.transactions_mut() = txs;
    block
}

pub fn generate_test_tx() -> Mint {
    let tx = Transaction::default_test_tx();
    let x = tx.as_mint().cloned();
    x.unwrap()
}

pub fn generate_test_data() -> MyTestData {
    MyTestData {
        ids: vec!["1".to_string(), "2".to_string(), "3".to_string()],
        version: 1u64,
        receipts: vec![
            "receipt_1".to_string(),
            "receipt_2".to_string(),
            "receipt_3".to_string(),
        ],
        assets: vec![AssetId::zeroed()],
        chain_id: ChainId::new(1),
    }
}

// Function to perform serialization and compression based on parameters
pub async fn perform_serialization(
    test_data: &MyTestData,
    serialization_type: SerializationType,
    compression_type: CompressionType,
    compression_level: Level,
) {
    let data_parser = DataParserBuilder::new()
        .with_compression(compression_type)
        .with_compression_level(compression_level)
        .with_serialization(serialization_type)
        .build();

    for _ in 0..10_000 {
        let _ = data_parser.serialize_and_compress(test_data).await.unwrap();
    }
}
