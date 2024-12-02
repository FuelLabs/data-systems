use criterion::{criterion_group, criterion_main, Criterion};
use load_tester::runners::runner_load_tester::run_blocks_consumer;
use nats_publisher::utils::nats::NatsHelper;
use tokio::runtime::Runtime;

static MSGS_LIMIT: usize = 10000;

fn benchmark_all(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let mut group = c.benchmark_group("NATS Benchmarks");
    let nats = rt.block_on(async { NatsHelper::connect(false).await.unwrap() });

    group.bench_function("consume_blocks_ack_none", |b| {
        b.to_async(&rt).iter(|| async {
            run_blocks_consumer(&nats, MSGS_LIMIT).await.unwrap()
        });
    });

    group.finish();
}

criterion_group!(
    name = benches;
    config = Criterion::default().sample_size(10);
    targets = benchmark_all
);
criterion_main!(benches);
