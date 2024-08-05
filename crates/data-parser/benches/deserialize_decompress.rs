use async_compression::Level;
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use data_parser::{
    builder::DataParserBuilder,
    generate_test_block,
    types::{CompressionType, SerializationType},
};
use fuel_core_types::{blockchain::block::Block, fuel_tx::Transaction};
use strum::IntoEnumIterator;

fn bench_decompress_deserialize(c: &mut Criterion) {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();

    // Pre-serialize data for each combination type
    let mut parametric_matrix = vec![];
    for serialization_type in SerializationType::iter() {
        for compression_type in CompressionType::iter() {
            for compression_level in
                [Level::Default, Level::Fastest, Level::Best]
            {
                let test_block = generate_test_block();
                let data_parser = DataParserBuilder::new()
                    .with_compression(compression_type)
                    .with_compression_level(compression_level)
                    .with_serialization(serialization_type)
                    .build();

                // Perform serialization asynchronously and collect the results
                let serialized_and_compressed = runtime.block_on(async {
                    data_parser
                        .serialize_and_compress(&test_block)
                        .await
                        .expect("serialization failed")
                });

                parametric_matrix.push((
                    serialization_type,
                    compression_type,
                    compression_level,
                    serialized_and_compressed,
                ));
            }
        }
    }

    let mut group = c.benchmark_group("decompress_deserialize");

    // benchmark each combination
    for (
        serialization_type,
        compression_type,
        compression_level,
        serialized_compressed_data,
    ) in parametric_matrix.iter()
    {
        let bench_name = format!(
            "[{:?}][{:?}][{:?}]",
            serialization_type.to_string(),
            compression_type.to_string(),
            compression_level
        );

        group.bench_function(&bench_name, |b| {
            let data_parser = DataParserBuilder::new()
                .with_compression(*compression_type)
                .with_compression_level(*compression_level)
                .with_serialization(*serialization_type)
                .build();

            b.to_async(&runtime).iter(|| async {
                let result = data_parser
                    .decompress_and_deserialize::<Block<Transaction>>(
                        serialized_compressed_data,
                    )
                    .await
                    .expect("decompresison and deserialization");
                // Use black_box to make sure 'result' is considered used by the compiler
                black_box(result);
            });
        });
    }

    group.finish();
}

criterion_group!(benches, bench_decompress_deserialize);
criterion_main!(benches);
