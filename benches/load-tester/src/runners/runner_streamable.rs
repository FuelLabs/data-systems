use std::sync::Arc;

use anyhow::Result;
use chrono::Utc;
use fuel_streams::{subjects::FromJsonString, Client, FuelNetwork};
use fuel_streams_core::{server::DeliverPolicy, subjects::IntoSubject};
use fuel_streams_store::record::Record;
use futures::StreamExt;
use tokio::task::JoinHandle;

use super::results::LoadTestTracker;

pub async fn run_streamable_consumer<
    S: IntoSubject + FromJsonString,
    T: Record,
>(
    network: FuelNetwork,
    api_key: String,
    subject: S,
    load_test_tracker: Arc<LoadTestTracker>,
) -> Result<()> {
    let mut client = Client::new(network).with_api_key(api_key);
    let mut connection = client.connect().await?;
    let mut stream = connection
        .subscribe::<T>(subject, DeliverPolicy::New)
        .await?;

    while let Some(msg) = stream.next().await {
        println!("Received entity: {:?}", msg.payload);
        load_test_tracker.increment_message_count();
        load_test_tracker
            .add_publish_time(Utc::now().timestamp_millis() as u128);
    }

    Ok(())
}

pub async fn spawn_streamable_consumer<
    S: IntoSubject + FromJsonString,
    T: Record,
>(
    network: FuelNetwork,
    api_key: String,
    subject: S,
    load_test_tracker: Arc<LoadTestTracker>,
) -> Result<JoinHandle<()>> {
    Ok(tokio::spawn(async move {
        if let Err(e) = run_streamable_consumer::<S, T>(
            network,
            api_key,
            subject.clone(),
            load_test_tracker,
        )
        .await
        {
            eprintln!(
                "Error in {:?} subscriptions - {:?}",
                subject.wildcard(),
                e
            );
        }
    }))
}
