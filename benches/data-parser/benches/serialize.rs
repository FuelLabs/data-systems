use criterion::{black_box, criterion_group, criterion_main, Criterion};
use data_parser::generate_test_block;
use fuel_data_parser::{
    DataParser,
    SerializationType,
    DEFAULT_COMPRESSION_STRATEGY,
};
use strum::IntoEnumIterator;

fn bench_serialize(c: &mut Criterion) {
    let mut group = c.benchmark_group("serialize");

    let test_block = generate_test_block();

    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();

    // Benchmarks for different serialization methods
    let parametric_matrix = SerializationType::iter()
        .map(|serialization_type| {
            (serialization_type, DEFAULT_COMPRESSION_STRATEGY.clone())
        })
        .collect::<Vec<_>>();

    for (serialization_type, compression_strategy) in parametric_matrix {
        let bench_name = format!("[{}]", serialization_type.to_string());

        group.bench_function(bench_name, |b| {
            let data_parser = DataParser::default()
                .with_compression_strategy(&compression_strategy)
                .with_serialization_type(serialization_type.clone());

            b.to_async(&runtime).iter(|| async {
                let result = data_parser
                    .serialize(&test_block)
                    .await
                    .expect("serialization");
                // Use black_box to make sure 'result' is considered used by the compiler
                black_box(result.len()); // record size of the data
            });
        });
    }

    group.finish();
}

criterion_group!(benches, bench_serialize);
criterion_main!(benches);
