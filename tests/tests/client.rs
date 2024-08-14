use std::collections::HashSet;

use async_nats::connection::State;
use fuel_streams::prelude::*;
use fuel_streams_core::prelude::*;
use futures::TryStreamExt;
use futures_util::future::try_join_all;
use pretty_assertions::assert_eq;
use streams_tests::server_setup;

#[tokio::test]
async fn conn_streams_has_required_streams() -> BoxedResult<()> {
    let (client, streams) = server_setup().await.unwrap();
    let mut context_streams = client.jetstream().stream_names();

    let mut names = HashSet::new();
    while let Some(name) = context_streams.try_next().await? {
        names.insert(name);
    }

    streams.blocks.assert_has_nats_stream(&names).await;
    streams.transactions.assert_has_nats_stream(&names).await;

    Ok(())
}

#[tokio::test]
async fn fuel_streams_client_connection() -> BoxedResult<()> {
    let opts = ClientOpts::admin_opts(NATS_URL);
    let client = NatsClient::connect(opts).await?;
    assert_eq!(client.state(), State::Connected);
    let client = Client::with_opts(conn.client.opts.to_owned()).await?;
    assert_eq!(conn.state(), State::Connected);
    assert_eq!(client.conn.state(), State::Connected);
    Ok(())
}

#[tokio::test]
async fn multiple_client_connections() -> BoxedResult<()> {
    let opts = ClientOpts::admin_opts(NATS_URL);
    let client = NatsClient::connect(opts).await?;
    let tasks: Vec<_> = (0..100)
        .map(|_| {
            let opts = opts.clone();
            async move {
                let client = Client::with_opts(opts).await.unwrap();
                let nats_client = client.conn.nats_client();
                assert_eq!(nats_client.connection_state(), State::Connected);
                Ok::<(), NatsError>(())
            }
        })
        .collect();

    assert!(try_join_all(tasks).await.is_ok());
    Ok(())
}

#[tokio::test]
async fn public_user_cannot_create_streams() -> BoxedResult<()> {
    let opts = ClientOpts::public_opts(NATS_URL)
        .with_rdn_namespace()
        .with_timeout(1);
    let client = NatsClient::connect(opts).await?;

    assert!(client
        .jetstream
        .create_stream(types::NatsStreamConfig {
            name: "test_stream".into(),
            subjects: vec!["test.>".into()],
            ..Default::default()
        })
        .await
        .is_err());

    Ok(())
}

#[tokio::test]
async fn public_user_cannot_create_stores() -> BoxedResult<()> {
    let opts = ClientOpts::public_opts(NATS_URL)
        .with_rdn_namespace()
        .with_timeout(1);

    let client = NatsClient::connect(opts).await?;
    assert!(client
        .jetstream
        .create_key_value(types::KvStoreConfig {
            bucket: "test".into(),
            ..Default::default()
        })
        .await
        .is_err());

    Ok(())
}

#[tokio::test]
async fn public_user_can_access_streams_after_created() {
    let opts = ClientOpts::new(NATS_URL)
        .with_rdn_namespace()
        .with_timeout(1);

    let admin_opts = opts.clone().with_role(NatsUserRole::Admin);
    assert!(NatsClient::connect(admin_opts).await.is_ok());

    let public_opts = opts.clone().with_role(NatsUserRole::Public);
    assert!(NatsClient::connect(public_opts).await.is_ok());
}
