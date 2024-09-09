use std::sync::Arc;

use chrono::Utc;
use fuel_core::database::database_description::DatabaseHeight;
use fuel_streams::types::ChainId;
use fuel_streams_core::{
    blocks::BlocksSubject,
    types::{Address, Block, BlockHeight, Transaction},
    Stream,
};
use fuel_streams_macros::subject::IntoSubject;
use tracing::info;

use crate::metrics::PublisherMetrics;

pub async fn publish(
    metrics: &Arc<PublisherMetrics>,
    chain_id: &ChainId,
    block_height: &BlockHeight,
    blocks_stream: &Stream<Block>,
    block: &Block<Transaction>,
    block_producer: &Address,
) -> anyhow::Result<()> {
    let block_subject: BlocksSubject = BlocksSubject::new()
        .with_height(Some(block_height.clone()))
        .with_producer(Some(block_producer.clone()));

    info!("NATS Publisher: Publishing Block #{block_height}");

    let published_data_size =
        match blocks_stream.publish(&block_subject, block).await {
            Ok(published_data_size) => published_data_size,
            Err(e) => {
                metrics.error_rates.with_label_values(&[
                    &chain_id.to_string(),
                    &block_producer.to_string(),
                    BlocksSubject::WILDCARD,
                    &e.to_string(),
                ]);
                return Err(anyhow::anyhow!(e));
            }
        };

    // update metrics
    metrics
        .message_size_histogram
        .with_label_values(&[
            &chain_id.to_string(),
            &block_producer.to_string(),
            BlocksSubject::WILDCARD,
        ])
        .observe(published_data_size as f64);

    let latency = block.header().time().to_unix() - Utc::now().timestamp();
    metrics
        .publishing_latency_histogram
        .with_label_values(&[
            &chain_id.to_string(),
            &block_producer.to_string(),
            BlocksSubject::WILDCARD,
        ])
        .observe(latency as f64);

    metrics
        .last_published_block_timestamp
        .with_label_values(&[
            &chain_id.to_string(),
            &block_producer.to_string(),
        ])
        .set(block.header().time().to_unix());

    metrics
        .last_published_block_height
        .with_label_values(&[
            &chain_id.to_string(),
            &block_producer.to_string(),
        ])
        .set(block.header().consensus().height.as_u64() as i64);

    metrics
        .total_published_messages
        .with_label_values(&[
            &chain_id.to_string(),
            &block_producer.to_string(),
        ])
        .inc();

    metrics
        .published_messages_throughput
        .with_label_values(&[
            &chain_id.to_string(),
            &block_producer.to_string(),
            BlocksSubject::WILDCARD,
        ])
        .inc();

    Ok(())
}
