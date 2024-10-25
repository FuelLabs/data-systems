use std::sync::Arc;

use chrono::Utc;
use fuel_core::database::database_description::DatabaseHeight;
use fuel_streams_core::prelude::*;
use futures::{StreamExt, TryStreamExt};

use crate::{
    metrics::PublisherMetrics,
    PublishError,
    PublishPayload,
    SubjectPayload,
    CONCURRENCY_LIMIT,
};

pub async fn publish_tasks(
    stream: &Stream<Block>,
    block: &Block<Transaction>,
    chain_id: &ChainId,
    block_producer: &Address,
    block_height: &BlockHeight,
    metrics: &Arc<PublisherMetrics>,
) -> Result<(), PublishError> {
    let payloads = create_publish_payloads(block, block_height, block_producer);

    add_metrics(chain_id, block, block_producer, metrics);
    futures::stream::iter(payloads)
        .map(Ok)
        .try_for_each_concurrent(*CONCURRENCY_LIMIT, |payload| {
            let metrics = metrics.clone();
            let chain_id = chain_id.to_owned();
            let block_producer = block_producer.clone();
            async move {
                payload
                    .publish(stream, &metrics, &chain_id, &block_producer)
                    .await
            }
        })
        .await
}

fn create_publish_payloads(
    block: &Block<Transaction>,
    block_height: &BlockHeight,
    block_producer: &Address,
) -> Vec<PublishPayload<Block>> {
    let subject: SubjectPayload = (
        BlocksSubject::new()
            .with_height(Some(block_height.clone()))
            .with_producer(Some(block_producer.clone()))
            .boxed(),
        BlocksSubject::WILDCARD,
    );

    vec![PublishPayload {
        subject,
        payload: block.to_owned(),
    }]
}

fn add_metrics(
    chain_id: &ChainId,
    block: &Block<Transaction>,
    block_producer: &Address,
    metrics: &Arc<PublisherMetrics>,
) -> Arc<PublisherMetrics> {
    let latency = Utc::now().timestamp() - block.header().time().to_unix();

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

    metrics.to_owned()
}
