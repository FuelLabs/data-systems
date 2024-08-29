use criterion::{black_box, criterion_group, criterion_main, Criterion};
use data_parser::generate_test_block;
use fuel_data_parser::{
    DataParser,
    SerializationType,
    ALL_COMPRESSION_STRATEGIES,
};
use strum::IntoEnumIterator;

fn bench_serialize_compress(c: &mut Criterion) {
    let mut group = c.benchmark_group("serialize_compress");

    let test_block = generate_test_block();

    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();

    // build test matrix
    for serialization_type in SerializationType::iter() {
        for compression_strategy in ALL_COMPRESSION_STRATEGIES.iter() {
            let bench_name = format!(
                "[{:?}][{:?}]",
                serialization_type.to_string(),
                compression_strategy.name(),
            );

            group.bench_function(&bench_name, |b| {
                let data_parser = DataParser::default()
                    .with_serialization_type(serialization_type.clone())
                    .with_compression_strategy(compression_strategy);

                b.to_async(&runtime).iter(|| async {
                    let result = data_parser
                        .encode(&test_block)
                        .await
                        .expect("serialization and compression error");
                    // Use black_box to make sure 'result' is considered used by the compiler
                    black_box(result.len()); // record size of the data
                });
            });
        }
    }

    group.finish();
}

criterion_group!(benches, bench_serialize_compress);
criterion_main!(benches);
