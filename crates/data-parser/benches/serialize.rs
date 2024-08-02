use async_compression::Level;
use criterion::{criterion_group, criterion_main, Criterion};
use data_parser::{
    generate_test_data,
    perform_serialization,
    types::{CompressionType, SerializationType},
};

fn bench_serialize(c: &mut Criterion) {
    let mut group = c.benchmark_group("serialize");

    let test_data = generate_test_data();

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
