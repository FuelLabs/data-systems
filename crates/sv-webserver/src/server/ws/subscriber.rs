use std::sync::Arc;

use fuel_streams_core::{nats::*, stream::*};
use fuel_streams_store::{
    db::{Db, DbRecord},
    record::RecordEntity,
    store::StoreResult,
};
use futures::{stream::BoxStream, StreamExt};

use super::models::DeliverPolicy;

pub async fn create_live_subscriber(
    record_entity: &RecordEntity,
    streams: &FuelStreams,
    subject_wildcard: String,
) -> Result<BoxStream<'static, StoreResult<DbRecord>>, StreamError> {
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
        RecordEntity::Log => {
            streams.logs.subscribe_live(subject_wildcard).await?
        }
    };
    Ok(stream)
}

pub async fn create_historical_subscriber(
    record_entity: &RecordEntity,
    streams: &FuelStreams,
    subject_wildcard: String,
) -> Result<BoxStream<'static, StoreResult<DbRecord>>, StreamError> {
    let historical = match record_entity {
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
        RecordEntity::Log => {
            streams.logs.subscribe_historical(subject_wildcard).await?
        }
    };
    Ok(Box::pin(historical.map(Ok)))
}

pub async fn create_subscriber(
    record_entity: &RecordEntity,
    nats_client: &Arc<NatsClient>,
    db: &Arc<Db>,
    subject_wildcard: String,
    deliver_policy: DeliverPolicy,
) -> Result<BoxStream<'static, StoreResult<DbRecord>>, StreamError> {
    let streams = FuelStreams::new(nats_client, db).await;

    if deliver_policy == DeliverPolicy::All {
        create_historical_subscriber(record_entity, &streams, subject_wildcard)
            .await
    } else {
        create_live_subscriber(record_entity, &streams, subject_wildcard).await
    }
}
