use criterion::{black_box, criterion_group, criterion_main, Criterion};
use data_parser::generate_test_block;
use fuel_core_types::{blockchain::block::Block, fuel_tx::Transaction};
use fuel_data_parser::{
    DataParser,
    SerializationType,
    ALL_COMPRESSION_STRATEGIES,
};
use strum::IntoEnumIterator;

fn bench_decompress_deserialize(c: &mut Criterion) {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();

    // Pre-serialize data for each combination type
    let mut parametric_matrix = vec![];
    for serialization_type in SerializationType::iter() {
        for compression_strategy in ALL_COMPRESSION_STRATEGIES.iter() {
            let data_parser = DataParser::default()
                .with_serialization_type(serialization_type.clone())
                .with_compression_strategy(compression_strategy);

            let serialized_and_compressed = runtime.block_on(async {
                data_parser
                    .encode(&generate_test_block())
                    .await
                    .expect("serialization failed")
            });

            parametric_matrix.push((
                serialization_type.clone(),
                compression_strategy,
                serialized_and_compressed,
            ));
        }
    }

    let mut group = c.benchmark_group("decompress_deserialize");

    // benchmark each combination
    for (serialization_type, compression_strategy, serialized_and_compressed) in
        parametric_matrix.iter()
    {
        let bench_name = format!(
            "[{:?}][{:?}]",
            serialization_type,
            compression_strategy.name(),
        );

        group.bench_function(&bench_name, |b| {
            let data_parser = DataParser::default()
                .with_compression_strategy(compression_strategy)
                .with_serialization_type(serialization_type.clone());

            b.to_async(&runtime).iter(|| async {
                let deserialized_and_decompressed = data_parser
                    .decode::<Block<Transaction>>(&serialized_and_compressed)
                    .await
                    .expect("decompresison and deserialization");

                // Use black_box to make sure 'result' is considered used by the compiler
                black_box(deserialized_and_decompressed);
            });
        });
    }

    group.finish();
}

criterion_group!(benches, bench_decompress_deserialize);
criterion_main!(benches);
