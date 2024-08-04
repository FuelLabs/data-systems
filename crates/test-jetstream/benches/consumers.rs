use std::error::Error;

use async_nats::jetstream::consumer::{pull::Config, AckPolicy, PullConsumer};
use criterion::{criterion_group, criterion_main, Criterion};
use fuel_core_types::{blockchain::block::Block, fuel_tx::Transaction};
use futures_util::StreamExt;
use test_jetstream::utils::nats::NatsHelper;
use tokio::runtime::Runtime;

async fn consume_messages_encoded(
    consumer: &PullConsumer,
    num_messages: usize,
) -> Result<(), Box<dyn Error>> {
    let mut messages = consumer.messages().await.unwrap();
    for _ in 0..num_messages {
        if let Some(message) = messages.next().await {
            let message = message.unwrap();
            let subject = message.subject.clone();
            if subject.contains("blocks") {
                let _: Block = bincode::deserialize(&message.payload)?;
            }
            if subject.contains("transactions") {
                let _: Transaction = bincode::deserialize(&message.payload)?;
            }
        }
    }
    Ok(())
}

async fn consume_messages_json(
    consumer: &PullConsumer,
    num_messages: usize,
) -> Result<(), Box<dyn Error>> {
    let mut messages = consumer.messages().await.unwrap();
    for _ in 0..num_messages {
        if let Some(message) = messages.next().await {
            let message = message.unwrap();
            let subject = message.subject.clone();
            if subject.contains("blocks") {
                let _: Block = serde_json::from_slice(&message.payload)?;
            }
            if subject.contains("transactions") {
                let _: Transaction = serde_json::from_slice(&message.payload)?;
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

    // Encoded consumers
    let blocks_encoded_consumer = rt.block_on(async {
        nats_helper
            .st_blocks_encoded
            .create_consumer(Config::default())
            .await
            .unwrap()
    });

    let blocks_encoded_consumer_ack_none = rt.block_on(async {
        nats_helper
            .st_blocks_encoded
            .create_consumer(Config {
                ack_policy: AckPolicy::None,
                ..Default::default()
            })
            .await
            .unwrap()
    });

    let transactions_encoded_consumer = rt.block_on(async {
        nats_helper
            .st_transactions_encoded
            .create_consumer(Config::default())
            .await
            .unwrap()
    });

    let transactions_encoded_consumer_ack_none = rt.block_on(async {
        nats_helper
            .st_transactions_encoded
            .create_consumer(Config {
                ack_policy: AckPolicy::None,
                ..Default::default()
            })
            .await
            .unwrap()
    });

    // JSON consumers
    let blocks_json_consumer = rt.block_on(async {
        nats_helper
            .st_blocks_json
            .create_consumer(Config::default())
            .await
            .unwrap()
    });

    let blocks_json_consumer_ack_none = rt.block_on(async {
        nats_helper
            .st_blocks_json
            .create_consumer(Config {
                ack_policy: AckPolicy::None,
                ..Default::default()
            })
            .await
            .unwrap()
    });

    let transactions_json_consumer = rt.block_on(async {
        nats_helper
            .st_transactions_json
            .create_consumer(Config::default())
            .await
            .unwrap()
    });

    let transactions_json_consumer_ack_none = rt.block_on(async {
        nats_helper
            .st_transactions_json
            .create_consumer(Config {
                ack_policy: AckPolicy::None,
                ..Default::default()
            })
            .await
            .unwrap()
    });

    // Benchmarks for encoded consumers
    c.bench_function("consume_blocks_encoded_20", |b| {
        b.to_async(&rt)
            .iter(|| consume_messages_encoded(&blocks_encoded_consumer, 20));
    });

    c.bench_function("consume_blocks_encoded_ack_none_20", |b| {
        b.to_async(&rt).iter(|| {
            consume_messages_encoded(&blocks_encoded_consumer_ack_none, 20)
        });
    });

    c.bench_function("consume_transactions_encoded_20", |b| {
        b.to_async(&rt).iter(|| {
            consume_messages_encoded(&transactions_encoded_consumer, 20)
        });
    });

    c.bench_function("consume_transactions_ack_none_encoded_20", |b| {
        b.to_async(&rt).iter(|| {
            consume_messages_encoded(
                &transactions_encoded_consumer_ack_none,
                20,
            )
        });
    });

    // Benchmarks for JSON consumers
    c.bench_function("consume_blocks_json_20", |b| {
        b.to_async(&rt)
            .iter(|| consume_messages_json(&blocks_json_consumer, 20));
    });

    c.bench_function("consume_blocks_json_20", |b| {
        b.to_async(&rt)
            .iter(|| consume_messages_json(&blocks_json_consumer_ack_none, 20));
    });

    c.bench_function("consume_transactions_json_20", |b| {
        b.to_async(&rt)
            .iter(|| consume_messages_json(&transactions_json_consumer, 20));
    });

    c.bench_function("consume_transactions_json_20", |b| {
        b.to_async(&rt).iter(|| {
            consume_messages_json(&transactions_json_consumer_ack_none, 20)
        });
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
