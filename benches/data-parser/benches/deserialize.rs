use criterion::{black_box, criterion_group, criterion_main, Criterion};
use data_parser::generate_test_block;
use fuel_core_types::{blockchain::block::Block, fuel_tx::Transaction};
use fuel_data_parser::{
    DataParser,
    SerializationType,
    DEFAULT_COMPRESSION_STRATEGY,
};
use strum::IntoEnumIterator;

fn bench_deserialize(c: &mut Criterion) {
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

    // Pre-serialize data for each serialization type
    let serialized_data: Vec<_> = parametric_matrix
        .iter()
        .map(|(serialization_type, compression_strategy)| {
            let test_block = generate_test_block();
            let data_parser = DataParser::default()
                .with_compression_strategy(compression_strategy)
                .with_serialization_type(serialization_type.clone());

            // Perform serialization asynchronously and collect the results
            let serialized = runtime.block_on(async {
                data_parser
                    .serialize(&test_block)
                    .await
                    .expect("serialization failed")
            });

            (serialization_type.clone(), compression_strategy, serialized)
        })
        .collect();

    let mut group = c.benchmark_group("deserialize");

    // benchmark each combination
    for (serialization_type, compression_strategy, serialized) in
        serialized_data
    {
        let bench_name = format!("[{}]", serialization_type);
        group.bench_function(&bench_name, |b| {
            let data_parser = DataParser::default()
                .with_compression_strategy(compression_strategy)
                .with_serialization_type(serialization_type.clone());

            b.iter(|| {
                // Perform deserialization
                let result = runtime.block_on(async {
                    data_parser
                        .deserialize::<Block<Transaction>>(&serialized)
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
