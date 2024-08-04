use std::error::Error;

use async_nats::jetstream::kv::Store;
use criterion::{criterion_group, criterion_main, Criterion};
use fuel_core_types::{blockchain::block::Block, fuel_tx::Transaction};
use futures_util::StreamExt;
use test_jetstream::utils::nats::NatsHelper;
use tokio::runtime::Runtime;

async fn watch_kv(
    store: &Store,
    subjects: &'static str,
    num_changes: usize,
) -> Result<(), Box<dyn Error>> {
    let mut watch = store.watch(subjects).await.unwrap();
    for _ in 0..num_changes {
        let entry = watch.next().await;
        if let Some(item) = entry {
            let item = item?;
            let subject = item.key;
            if subject.contains("blocks") {
                let _: Block = bincode::deserialize(&item.value)?;
            }
            if subject.contains("transactions") {
                let _: Transaction = bincode::deserialize(&item.value)?;
            }
        }
    }
    Ok(())
}

fn criterion_benchmark(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    let nats_helper = rt.block_on(async {
        NatsHelper::connect()
            .await
            .expect("Failed to connect to NATS")
    });

    // Benchmark for watching blocks KV store
    c.bench_function("watch_kv_blocks_20", |b| {
        b.to_async(&rt)
            .iter(|| watch_kv(&nats_helper.kv_blocks, "blocks.>", 20));
    });

    // Benchmark for watching transactions KV store
    c.bench_function("watch_kv_transactions_20", |b| {
        b.to_async(&rt)
            .iter(|| watch_kv(&nats_helper.kv_transactions, "blocks.>", 20));
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
