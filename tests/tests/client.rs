use std::collections::HashSet;

use async_nats::connection::State;
use fuel_streams::prelude::*;
use fuel_streams_core::prelude::*;
use futures::TryStreamExt;
use futures_util::future::try_join_all;
use pretty_assertions::assert_eq;

#[tokio::test]
async fn conn_has_all_stores_by_default() -> BoxedResult<()> {
    let opts = ClientOpts::admin_opts(NATS_URL);
    let conn = NatsConn::connect(opts).await?;
    let store_list = conn.stores().get_store_list();
    let mut context_streams = conn.jetstream().stream_names();

    let mut names = HashSet::new();
    while let Some(name) = context_streams.try_next().await? {
        names.insert(name);
    }

    for store in store_list {
        assert!(
            names.contains(&store.stream_name),
            "store {} not found in JetStream",
            store.stream_name
        );
    }

    Ok(())
}

#[tokio::test]
async fn fuel_streams_client_connection() -> BoxedResult<()> {
    // We need to first connect as admin to have the streams created by first hand
    // since public users doesnt have permissions to
    let opts = ClientOpts::admin_opts(NATS_URL);
    let conn = NatsConn::connect(opts).await?;
    let client = Client::with_opts(conn.client.opts.to_owned()).await?;
    assert_eq!(conn.state(), State::Connected);
    assert_eq!(client.conn.state(), State::Connected);
    Ok(())
}

#[tokio::test]
async fn multiple_client_connections() -> BoxedResult<()> {
    let opts = ClientOpts::admin_opts(NATS_URL);
    let conn = NatsConn::connect(opts).await?;
    let tasks: Vec<_> = (0..100)
        .map(|_| {
            let conn = conn.clone();
            async move {
                let client = Client::with_opts(conn.client.opts).await?;
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
    assert!(client.create_store("test", None).await.is_err());

    Ok(())
}

#[tokio::test]
async fn public_user_can_access_stores_after_created() {
    let opts = ClientOpts::new(NATS_URL)
        .with_rdn_namespace()
        .with_timeout(1);

    let admin_opts = opts.clone().with_role(NatsUserRole::Admin);
    assert!(NatsConn::connect(admin_opts).await.is_ok());

    let public_opts = opts.clone().with_role(NatsUserRole::Public);
    assert!(NatsConn::connect(public_opts).await.is_ok());
}
