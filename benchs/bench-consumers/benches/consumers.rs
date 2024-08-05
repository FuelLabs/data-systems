use bench_consumers::runners::{
    runner_consumer::run_consume_blocks_encoded,
    runner_kv_watcher::run_watch_kv_blocks,
    runner_subscription::run_subscriptions,
};
use criterion::{criterion_group, criterion_main, Criterion};
use tokio::runtime::Runtime;

fn criterion_benchmark(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    c.bench_function("run_consume_blocks_encoded", |b| {
        b.to_async(&rt).iter(|| async {
            let result = run_consume_blocks_encoded().await.unwrap();
            result.print_result();
        });
    });

    c.bench_function("run_watch_kv_blocks", |b| {
        b.to_async(&rt).iter(|| async {
            let result = run_watch_kv_blocks().await.unwrap();
            result.print_result();
        });
    });

    c.bench_function("run_subscriptions", |b| {
        b.to_async(&rt).iter(|| async {
            let result = run_subscriptions().await.unwrap();
            result.print_result();
        });
    });
}

criterion_group! {
    name = benches;
    config = Criterion::default()
        .sample_size(10)
        .measurement_time(std::time::Duration::from_secs(60));
    targets = criterion_benchmark
}
criterion_main!(benches);
