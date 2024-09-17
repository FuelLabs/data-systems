use std::sync::Arc;

use chrono::Utc;
use fuel_core::database::database_description::DatabaseHeight;
use fuel_streams_core::{
    blocks::BlocksSubject,
    prelude::*,
    types::{Address, Block, BlockHeight, Transaction},
    Stream,
};
use tracing::info;

use crate::{metrics::PublisherMetrics, publish_with_metrics, FuelCoreLike};

pub async fn publish(
    metrics: &Arc<PublisherMetrics>,
    fuel_core: &dyn FuelCoreLike,
    blocks_stream: &Stream<Block>,
    block: &Block<Transaction>,
    block_producer: &Address,
) -> anyhow::Result<()> {
    let chain_id = fuel_core.chain_id();
    let block_height: BlockHeight = block.header().consensus().height.into();
    let block_subject: BlocksSubject = BlocksSubject::new()
        .with_height(Some(block_height.clone()))
        .with_producer(Some(block_producer.clone()));

    info!("NATS Publisher: Publishing Block #{block_height}");

    publish_with_metrics!(
        blocks_stream.publish(&block_subject, block),
        metrics,
        chain_id,
        block_producer,
        BlocksSubject::WILDCARD
    );

    let latency = Utc::now().timestamp() - block.header().time().to_unix();
    metrics
        .publishing_latency_histogram
        .with_label_values(&[
            &fuel_core.chain_id().to_string(),
            &block_producer.to_string(),
            BlocksSubject::WILDCARD,
        ])
        .observe(latency as f64);

    metrics
        .last_published_block_timestamp
        .with_label_values(&[
            &fuel_core.chain_id().to_string(),
            &block_producer.to_string(),
        ])
        .set(block.header().time().to_unix());

    metrics
        .last_published_block_height
        .with_label_values(&[
            &fuel_core.chain_id().to_string(),
            &block_producer.to_string(),
        ])
        .set(block.header().consensus().height.as_u64() as i64);

    Ok(())
}
