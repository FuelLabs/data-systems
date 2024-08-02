use async_compression::Level;
use criterion::{criterion_group, criterion_main, Criterion};
use data_parser::{
    generate_test_data,
    perform_serialization,
    types::{CompressionType, SerializationType},
};
use strum::IntoEnumIterator;

fn bench_serialize_compress(c: &mut Criterion) {
    let mut group = c.benchmark_group("serialize_compress");

    let test_data = generate_test_data();

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
        }
    }

    group.finish();
}

criterion_group!(benches, bench_serialize_compress);
criterion_main!(benches);
