use std::sync::Arc;

use anyhow::Result;
use chrono::Utc;
use fuel_streams::{Client, FuelNetwork, SubjectPayload};
use fuel_streams_core::{server::DeliverPolicy, subjects::IntoSubject};
use futures::StreamExt;
use tokio::task::JoinHandle;

use super::results::LoadTestTracker;

pub async fn run_streamable_consumer(
    network: FuelNetwork,
    api_key: String,
    subject: SubjectPayload,
    load_test_tracker: Arc<LoadTestTracker>,
) -> Result<()> {
    let mut client = Client::new(network).with_api_key(api_key);
    let mut connection = client.connect().await?;
    let subjects = vec![subject.into()];
    let mut stream = connection.subscribe(subjects, DeliverPolicy::New).await?;
    while let Some(msg) = stream.next().await {
        let msg = msg?;
        println!("Received entity: {:?}", msg.payload);
        load_test_tracker.increment_message_count();
        load_test_tracker
            .add_publish_time(Utc::now().timestamp_millis() as u128);
    }

    Ok(())
}

pub async fn spawn_streamable_consumer<S: IntoSubject + Clone>(
    network: FuelNetwork,
    api_key: String,
    subject: S,
    load_test_tracker: Arc<LoadTestTracker>,
) -> Result<JoinHandle<()>> {
    Ok(tokio::spawn(async move {
        if let Err(e) = run_streamable_consumer(
            network,
            api_key,
            subject.clone().into(),
            load_test_tracker,
        )
        .await
        {
            eprintln!("Error in {:?} subscriptions - {:?}", subject.parse(), e);
        }
    }))
}
