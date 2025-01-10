use std::sync::Arc;

use fuel_streams_core::stream::*;
use fuel_streams_domains::SubjectPayload;
use fuel_streams_store::record::RecordEntity;

use super::errors::WsSubscriptionError;

pub async fn create_subscriber(
    streams: &Arc<FuelStreams>,
    subject_json: &SubjectPayload,
    deliver_policy: DeliverPolicy,
) -> Result<BoxedStream, WsSubscriptionError> {
    let record_entity = subject_json.record_entity();
    let stream = match record_entity {
        RecordEntity::Block => {
            let subject = subject_json.into_subject();
            streams
                .blocks
                .subscribe_dynamic(subject, deliver_policy)
                .await
        }
        RecordEntity::Transaction => {
            let subject = subject_json.into_subject();
            streams
                .transactions
                .subscribe_dynamic(subject, deliver_policy)
                .await
        }
        RecordEntity::Input => {
            let subject = subject_json.into_subject();
            streams
                .inputs
                .subscribe_dynamic(subject, deliver_policy)
                .await
        }
        RecordEntity::Output => {
            let subject = subject_json.into_subject();
            streams
                .outputs
                .subscribe_dynamic(subject, deliver_policy)
                .await
        }
        RecordEntity::Receipt => {
            let subject = subject_json.into_subject();
            streams
                .receipts
                .subscribe_dynamic(subject, deliver_policy)
                .await
        }
        RecordEntity::Utxo => {
            let subject = subject_json.into_subject();
            streams
                .utxos
                .subscribe_dynamic(subject, deliver_policy)
                .await
        }
    };
    Ok(Box::new(stream))
}
