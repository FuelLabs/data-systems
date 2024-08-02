use async_compression::Level;
use criterion::{criterion_group, criterion_main, Criterion};
use data_parser::{
    builder::DataParserBuilder,
    types::{CompressionType, SerializationType},
};
use fuel_core_types::{fuel_tx::AssetId, fuel_types::ChainId};

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, Eq, PartialEq)]
struct MyTestData {
    ids: Vec<String>,
    version: u64,
    receipts: Vec<String>,
    assets: Vec<AssetId>,
    chain_id: ChainId,
}

// Function to perform serialization and compression based on parameters
async fn perform_serialization(
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

fn bench_serialize(c: &mut Criterion) {
    let mut group = c.benchmark_group("serialize");

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

    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();

    // Benchmarks for different serialization methods
    let parametric_matrix = vec![
        (
            SerializationType::Bincode,
            CompressionType::None,
            Level::Default,
        ),
        (
            SerializationType::Postcard,
            CompressionType::None,
            Level::Default,
        ),
        (
            SerializationType::Json,
            CompressionType::None,
            Level::Default,
        ),
    ];

    for (serialization_type, compression_type, compression_level) in
        parametric_matrix
    {
        let bench_name = format!("[{}]", serialization_type.to_string());
        group.bench_function(bench_name, |b| {
            b.to_async(&runtime).iter(|| {
                perform_serialization(
                    &test_data,
                    serialization_type,
                    compression_type,
                    compression_level,
                )
            });
        });
    }

    group.finish();
}

criterion_group!(benches, bench_serialize);
criterion_main!(benches);
