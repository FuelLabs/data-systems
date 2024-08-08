use async_compression::Level;
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use data_parser::generate_test_block;
use fuel_data_parser::{CompressionType, DataParserBuilder, SerializationType};
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
        for compression_type in CompressionType::iter() {
            for compression_level in
                [Level::Default, Level::Fastest, Level::Best]
            {
                let bench_name = format!(
                    "[{:?}][{:?}][{:?}]",
                    serialization_type.to_string(),
                    compression_type.to_string(),
                    compression_level
                );

                group.bench_function(&bench_name, |b| {
                    let data_parser = DataParserBuilder::new()
                        .with_compression(compression_type)
                        .with_compression_level(compression_level)
                        .with_serialization(serialization_type)
                        .build();

                    b.to_async(&runtime).iter(|| async {
                        let result = data_parser
                            .test_serialize(&test_block)
                            .await
                            .expect("serialization");
                        // Use black_box to make sure 'result' is considered used by the compiler
                        black_box(result.len()); // record size of the data
                    });
                });
            }
        }
    }

    group.finish();
}

criterion_group!(benches, bench_serialize_compress);
criterion_main!(benches);
