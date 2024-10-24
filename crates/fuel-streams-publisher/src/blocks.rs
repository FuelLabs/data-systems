use std::sync::Arc;

use chrono::Utc;
use fuel_core::database::database_description::DatabaseHeight;
use fuel_streams_core::prelude::*;

use crate::{metrics::PublisherMetrics, PublishPayload, SubjectPayload};

pub fn create_publish_payloads(
    stream: &Stream<Block>,
    block: &Block<Transaction>,
    block_height: BlockHeight,
    block_producer: &Address,
) -> Vec<PublishPayload<Block>> {
    let subjects: Vec<SubjectPayload> = vec![(
        BlocksSubject::new()
            .with_height(Some(block_height.clone()))
            .with_producer(Some(block_producer.clone()))
            .boxed(),
        BlocksSubject::WILDCARD,
    )];

    vec![PublishPayload {
        subjects,
        stream: stream.to_owned(),
        payload: block.to_owned(),
    }]
}

pub fn add_metrics(
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
