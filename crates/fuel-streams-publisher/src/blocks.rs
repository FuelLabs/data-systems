use std::sync::Arc;

use chrono::Utc;
use fuel_core::database::database_description::DatabaseHeight;
use fuel_streams_core::prelude::*;
use tracing::info;

use crate::{
    metrics::PublisherMetrics,
    publish_all,
    FuelCoreLike,
    PublishPayload,
};

pub async fn publish(
    metrics: &Arc<PublisherMetrics>,
    fuel_core: &dyn FuelCoreLike,
    stream: &Stream<Block>,
    block: &Block<Transaction>,
    block_producer: &Address,
) -> anyhow::Result<()> {
    let chain_id = fuel_core.chain_id();
    let block_height: BlockHeight = block.header().consensus().height.into();
    let subjects: Vec<(Box<dyn IntoSubject>, &'static str)> = vec![(
        BlocksSubject::new()
            .with_height(Some(block_height.clone()))
            .with_producer(Some(block_producer.clone()))
            .boxed(),
        BlocksSubject::WILDCARD,
    )];

    info!("NATS Publisher: Publishing Block #{block_height}");

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

    publish_all(PublishPayload {
        stream,
        subjects,
        payload: block,
        metrics,
        chain_id,
        block_producer,
    })
    .await;

    Ok(())
}
