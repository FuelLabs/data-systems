use async_compression::Level;
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use data_parser::generate_test_block;
use fuel_core_types::{blockchain::block::Block, fuel_tx::Transaction};
use fuel_data_parser::{CompressionType, DataParserBuilder, SerializationType};
use strum::IntoEnumIterator;

fn bench_deserialize(c: &mut Criterion) {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();

    // Benchmarks for different serialization methods
    let parametric_matrix = SerializationType::iter()
        .map(|ser_type| (ser_type, CompressionType::None, Level::Default))
        .collect::<Vec<_>>();

    // Pre-serialize data for each serialization type
    let serialized_data: Vec<(
        SerializationType,
        CompressionType,
        Level,
        Vec<u8>,
    )> = parametric_matrix
        .iter()
        .map(
            |(serialization_type, compression_type, compression_level)| {
                let test_block = generate_test_block();
                let data_parser = DataParserBuilder::new()
                    .with_compression(*compression_type)
                    .with_compression_level(*compression_level)
                    .with_serialization(*serialization_type)
                    .build();

                // Perform serialization asynchronously and collect the results
                let serialized = runtime.block_on(async {
                    data_parser
                        .test_serialize(&test_block)
                        .await
                        .expect("serialization failed")
                });

                (
                    *serialization_type,
                    *compression_type,
                    *compression_level,
                    serialized,
                )
            },
        )
        .collect();

    let mut group = c.benchmark_group("deserialize");

    // benchmark each combination
    for (serialization_type, compression_type, compression_level, serialized) in
        serialized_data
    {
        let bench_name = format!("[{}]", serialization_type.to_string());
        group.bench_function(&bench_name, |b| {
            let data_parser = DataParserBuilder::new()
                .with_compression(compression_type)
                .with_compression_level(compression_level)
                .with_serialization(serialization_type)
                .build();

            b.iter(|| {
                // Perform deserialization
                let result = runtime.block_on(async {
                    data_parser
                        .test_deserialize::<Block<Transaction>>(&serialized)
                        .await
                        .expect("deserialization failed")
                });
                // Use black_box to make sure 'result' is considered used by the compiler
                black_box(result);
            });
        });
    }

    group.finish();
}

criterion_group!(benches, bench_deserialize);
criterion_main!(benches);
