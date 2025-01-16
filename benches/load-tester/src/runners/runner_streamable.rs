use std::sync::Arc;

use anyhow::Result;
use chrono::Utc;
use fuel_streams_core::{subjects::IntoSubject, DeliverPolicy, FuelStreams};
use fuel_streams_store::record::DataEncoder;
use futures::StreamExt;

use super::results::LoadTestTracker;

pub async fn run_streamable_consumer<S: IntoSubject, T: DataEncoder>(
    subject: S,
    fuel_streams: Arc<FuelStreams>,
    load_test_tracker: Arc<LoadTestTracker>,
) -> Result<()> {
    let mut subscriber = fuel_streams
        .blocks
        .subscribe(subject, DeliverPolicy::New)
        .await
        .enumerate();

    while let Some((_index, record)) = subscriber.next().await {
        let record = record.unwrap();
        let _decoded_entity = T::decode(&record).await.unwrap();
        load_test_tracker.increment_message_count();
        load_test_tracker
            .add_publish_time(Utc::now().timestamp_millis() as u128);
    }

    Ok(())
}
