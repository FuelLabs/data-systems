use std::sync::Arc;

use fuel_streams_core::{
    server::DeliverPolicy,
    subjects::*,
    types::StreamResponse,
    StreamError,
};
use fuel_streams_test::{
    close_db,
    create_multiple_records,
    create_random_db_name,
    setup_stream,
};
use fuel_web_utils::api_key::{ApiKeyError, MockApiKeyRole};
use futures::StreamExt;
use pretty_assertions::assert_eq;

const NATS_URL: &str = "nats://localhost:4222";

#[tokio::test]
async fn test_streaming_live_data() -> anyhow::Result<()> {
    let prefix = create_random_db_name();
    let stream = setup_stream(NATS_URL, &prefix).await?;
    let data = create_multiple_records(10, 0, &prefix);

    tokio::spawn({
        let data = data.clone();
        let stream = stream.clone();
        async move {
            let subject = BlocksSubject::new().with_height(None);
            let role = MockApiKeyRole::admin().into_inner();
            let mut subscriber = stream
                .subscribe(subject, DeliverPolicy::New, &role)
                .await
                .enumerate();

            while let Some((index, record)) = subscriber.next().await {
                let record = record.unwrap();
                let expected_block = &data[index].1;
                let block = record.payload.as_block().unwrap();
                assert_eq!((*block).clone(), *expected_block);
                if index == data.len() - 1 {
                    break;
                }
            }
        }
    });

    for record in data {
        let packet = record.2.to_owned().with_namespace(&prefix);
        let subject = packet.subject_str();
        let response = StreamResponse::try_from(&packet)?;
        stream.publish(&subject, &Arc::new(response)).await?;
    }

    close_db(&stream.store().db).await;
    Ok(())
}

#[tokio::test]
async fn test_streaming_live_data_without_proper_role() -> anyhow::Result<()> {
    let prefix = create_random_db_name();
    let stream = setup_stream(NATS_URL, &prefix).await?;
    let role = MockApiKeyRole::no_scopes().into_inner();
    let subject = BlocksSubject::new().with_height(None);
    let mut subscriber =
        stream.subscribe(subject, DeliverPolicy::New, &role).await;

    let result = subscriber.next().await;
    assert!(result.is_some());
    let error = result.unwrap();
    assert!(error.is_err());

    if let Err(err) = error {
        assert!(matches!(
            err,
            StreamError::ApiKey(ApiKeyError::ScopePermission(_))
        ));
    }

    close_db(&stream.store().db).await;
    Ok(())
}
