use std::sync::Arc;

use fuel_streams_core::{nats::*, stream::*};
use fuel_streams_store::{db::Db, record::RecordEntity};

use super::models::DeliverPolicy;

/// Creates a live subscription stream for a given record entity
pub async fn create_live_subscriber(
    record_entity: &RecordEntity,
    streams: &FuelStreams,
    subject_wildcard: String,
) -> Result<BoxedStream, StreamError> {
    let stream = match record_entity {
        RecordEntity::Block => {
            streams.blocks.subscribe_live(subject_wildcard).await?
        }
        RecordEntity::Transaction => {
            streams
                .transactions
                .subscribe_live(subject_wildcard)
                .await?
        }
        RecordEntity::Input => {
            streams.inputs.subscribe_live(subject_wildcard).await?
        }
        RecordEntity::Output => {
            streams.outputs.subscribe_live(subject_wildcard).await?
        }
        RecordEntity::Receipt => {
            streams.receipts.subscribe_live(subject_wildcard).await?
        }
        RecordEntity::Utxo => {
            streams.utxos.subscribe_live(subject_wildcard).await?
        }
    };
    Ok(Box::new(stream))
}

/// Creates a historical subscription stream for a given record entity
pub async fn create_historical_subscriber(
    record_entity: &RecordEntity,
    streams: &FuelStreams,
    subject_wildcard: String,
) -> Result<BoxedStream, StreamError> {
    let stream = match record_entity {
        RecordEntity::Block => {
            streams
                .blocks
                .subscribe_historical(subject_wildcard)
                .await?
        }
        RecordEntity::Transaction => {
            streams
                .transactions
                .subscribe_historical(subject_wildcard)
                .await?
        }
        RecordEntity::Input => {
            streams
                .inputs
                .subscribe_historical(subject_wildcard)
                .await?
        }
        RecordEntity::Output => {
            streams
                .outputs
                .subscribe_historical(subject_wildcard)
                .await?
        }
        RecordEntity::Receipt => {
            streams
                .receipts
                .subscribe_historical(subject_wildcard)
                .await?
        }
        RecordEntity::Utxo => {
            streams.utxos.subscribe_historical(subject_wildcard).await?
        }
    };
    Ok(Box::new(stream))
}

/// Creates a subscription stream based on the deliver policy
pub async fn create_subscriber(
    record_entity: &RecordEntity,
    nats_client: &Arc<NatsClient>,
    db: &Arc<Db>,
    subject_wildcard: String,
    deliver_policy: DeliverPolicy,
) -> Result<BoxedStream, StreamError> {
    let streams = FuelStreams::new(nats_client, db).await;
    if deliver_policy == DeliverPolicy::All {
        create_historical_subscriber(record_entity, &streams, subject_wildcard)
            .await
    } else {
        create_live_subscriber(record_entity, &streams, subject_wildcard).await
    }
}
