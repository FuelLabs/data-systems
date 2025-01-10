use std::sync::Arc;

use fuel_streams_core::stream::*;
use fuel_streams_domains::SubjectPayload;
use fuel_streams_store::record::RecordEntity;

use super::errors::WsSubscriptionError;

pub async fn create_live_subscriber(
    streams: &Arc<FuelStreams>,
    subject_json: &SubjectPayload,
) -> Result<BoxedStream, WsSubscriptionError> {
    let record_entity = subject_json.record_entity();
    let stream = match record_entity {
        RecordEntity::Block => {
            let subject = subject_json.into_subject();
            streams.blocks.subscribe_live(subject).await
        }
        RecordEntity::Transaction => {
            let subject = subject_json.into_subject();
            streams.transactions.subscribe_live(subject).await
        }
        RecordEntity::Input => {
            let subject = subject_json.into_subject();
            streams.inputs.subscribe_live(subject).await
        }
        RecordEntity::Output => {
            let subject = subject_json.into_subject();
            streams.outputs.subscribe_live(subject).await
        }
        RecordEntity::Receipt => {
            let subject = subject_json.into_subject();
            streams.receipts.subscribe_live(subject).await
        }
        RecordEntity::Utxo => {
            let subject = subject_json.into_subject();
            streams.utxos.subscribe_live(subject).await
        }
    };
    Ok(Box::new(stream))
}

pub async fn create_historical_subscriber(
    streams: &Arc<FuelStreams>,
    subject_json: &SubjectPayload,
) -> Result<BoxedStream, WsSubscriptionError> {
    let record_entity = subject_json.record_entity();
    let stream = match record_entity {
        RecordEntity::Block => {
            let subject = subject_json.into_subject();
            streams.blocks.subscribe_historical(subject).await
        }
        RecordEntity::Transaction => {
            let subject = subject_json.into_subject();
            streams.transactions.subscribe_historical(subject).await
        }
        RecordEntity::Input => {
            let subject = subject_json.into_subject();
            streams.inputs.subscribe_historical(subject).await
        }
        RecordEntity::Output => {
            let subject = subject_json.into_subject();
            streams.outputs.subscribe_historical(subject).await
        }
        RecordEntity::Receipt => {
            let subject = subject_json.into_subject();
            streams.receipts.subscribe_historical(subject).await
        }
        RecordEntity::Utxo => {
            let subject = subject_json.into_subject();
            streams.utxos.subscribe_historical(subject).await
        }
    };
    Ok(Box::new(stream))
}
