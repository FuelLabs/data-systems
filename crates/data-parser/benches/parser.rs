use async_compression::Level;
use criterion::{
    async_executor::AsyncExecutor,
    black_box,
    criterion_group,
    criterion_main,
    Criterion,
};
use data_parser::{
    builder::DataParserBuilder,
    types::{CompressionType, SerializationType},
};
use fuel_core_types::{fuel_tx::AssetId, fuel_types::ChainId};
use tokio::runtime::Runtime;

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, Eq, PartialEq)]
struct MyTestData {
    ids: Vec<String>,
    version: u64,
    receipts: Vec<String>,
    assets: Vec<AssetId>,
    chain_id: ChainId,
}

fn bench_serialize(c: &mut Criterion) {
    let mut group = c.benchmark_group("serialize");

    // construct the data parser
    let data_parser = DataParserBuilder::new()
        .with_compression(CompressionType::Gzip)
        .with_compression_level(Level::Fastest)
        .with_serialization(SerializationType::Bincode)
        .build();

    let test_data = MyTestData {
        ids: vec!["1".to_string(), "2".to_string(), "3".to_string()],
        version: 1u64,
        receipts: vec![
            "receipt_1".to_string(),
            "receipt_2".to_string(),
            "receipt_3".to_string(),
        ],
        assets: vec![AssetId::zeroed()],
        chain_id: ChainId::new(1),
    };

    group.bench_function("push single deque", |b| {
        // b.to_async(Runtime).iter(|| {
        //     for i in 0..10_000 {
        //         let ser_compressed_data = data_parser.serialize_and_compress(&test_data);
        //         let my_test_data_recreated = data_parser.decompress_and_deserialize::<MyTestData>(&ser_compressed_data).unwrap();
        //     }
        // });
    });

    group.finish();
}

criterion_group!(benches, bench_serialize);
criterion_main!(benches);
