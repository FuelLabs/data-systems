use std::collections::HashSet;

use fuel_streams::prelude::*;
use fuel_streams_core::prelude::*;
use futures::{future::BoxFuture, FutureExt, TryStreamExt};
use futures_util::future::try_join_all;
use rand::{distributions::Alphanumeric, Rng};
use streams_tests::server_setup;

fn gen_random_string(size: usize) -> String {
    rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(size)
        .map(char::from)
        .collect()
}

#[tokio::test]
async fn conn_streams_has_required_streams() -> BoxedResult<()> {
    let (client, streams) = server_setup().await.unwrap();
    let mut context_streams = client.jetstream.stream_names();

    let mut names = HashSet::new();
    while let Some(name) = context_streams.try_next().await? {
        names.insert(name);
    }
    streams.blocks.assert_has_stream(&names).await;
    streams.transactions.assert_has_stream(&names).await;

    for name in names.iter() {
        let empty = streams.blocks.is_empty(name).await;
        assert!(empty, "stream must be empty after creation");
    }

    Ok(())
}

#[tokio::test]
async fn fuel_streams_client_connection() -> BoxedResult<()> {
    let opts = NatsClientOpts::admin_opts(NATS_URL);
    let client = NatsClient::connect(&opts).await?;
    assert!(client.is_connected());
    let client = Client::with_opts(&opts).await?;
    assert!(client.conn.is_connected());
    Ok(())
}

#[tokio::test]
async fn multiple_client_connections() -> BoxedResult<()> {
    let opts = NatsClientOpts::admin_opts(NATS_URL);
    let tasks: Vec<_> = (0..100)
        .map(|_| {
            let opts = opts.clone();
            async move {
                let client = Client::with_opts(&opts).await.unwrap();
                assert!(client.conn.is_connected());
                Ok::<(), NatsError>(())
            }
        })
        .collect();

    assert!(try_join_all(tasks).await.is_ok());
    Ok(())
}

#[tokio::test]
async fn public_user_cannot_create_streams() -> BoxedResult<()> {
    let opts = NatsClientOpts::public_opts(NATS_URL)
        .with_rdn_namespace()
        .with_timeout(1);
    let client = NatsClient::connect(&opts).await?;
    let (random_stream_title, random_subject) =
        (gen_random_string(6), gen_random_string(6));

    assert!(client
        .jetstream
        .create_stream(types::NatsStreamConfig {
            name: random_stream_title,
            subjects: vec![random_subject],
            ..Default::default()
        })
        .await
        .is_err());

    Ok(())
}

#[tokio::test]
async fn public_user_cannot_create_stores() -> BoxedResult<()> {
    let opts = NatsClientOpts::public_opts(NATS_URL)
        .with_rdn_namespace()
        .with_timeout(1);

    let random_bucket_title = gen_random_string(6);

    let client = NatsClient::connect(&opts).await?;
    assert!(client
        .jetstream
        .create_key_value(types::KvStoreConfig {
            bucket: random_bucket_title,
            ..Default::default()
        })
        .await
        .is_err());

    Ok(())
}

#[tokio::test]
async fn public_user_can_access_streams_after_created() {
    let opts = NatsClientOpts::new(NATS_URL)
        .with_rdn_namespace()
        .with_timeout(1);

    let admin_opts = opts.clone().with_role(NatsUserRole::Admin);
    assert!(NatsClient::connect(&admin_opts).await.is_ok());

    let public_opts = opts.clone().with_role(NatsUserRole::Public);
    assert!(NatsClient::connect(&public_opts).await.is_ok());
}

#[tokio::test]
async fn public_and_admin_user_can_access_streams_after_created(
) -> BoxedResult<()> {
    let admin_opts = NatsClientOpts::admin_opts(NATS_URL);
    let admin_tasks: Vec<BoxFuture<'_, Result<(), NatsError>>> = (0..100)
        .map(|_| {
            let opts: NatsClientOpts = admin_opts.clone();
            async move {
                let client = Client::with_opts(&opts).await.unwrap();
                assert!(client.conn.is_connected());
                Ok::<(), NatsError>(())
            }
            .boxed() // Convert the future to a BoxFuture
        })
        .collect();

    let public_opts = NatsClientOpts::public_opts(NATS_URL);
    let public_tasks: Vec<BoxFuture<'_, Result<(), NatsError>>> = (0..100)
        .map(|_| {
            let opts: NatsClientOpts = public_opts.clone();
            async move {
                let client = Client::with_opts(&opts).await.unwrap();
                assert!(client.conn.is_connected());
                Ok::<(), NatsError>(())
            }
            .boxed() // Convert the future to a BoxFuture
        })
        .collect();

    // Combine both vectors into one
    let mut all_tasks =
        Vec::with_capacity(admin_tasks.len() + public_tasks.len());
    all_tasks.extend(admin_tasks);
    all_tasks.extend(public_tasks);

    assert!(try_join_all(all_tasks).await.is_ok());
    Ok(())
}

#[tokio::test]
async fn admin_user_can_delete_stream() -> BoxedResult<()> {
    let opts = NatsClientOpts::admin_opts(NATS_URL)
        .with_rdn_namespace()
        .with_timeout(1);
    let client = NatsClient::connect(&opts).await?;

    let (random_stream_title, random_subject) =
        (gen_random_string(6), gen_random_string(6));

    client
        .jetstream
        .create_stream(types::NatsStreamConfig {
            name: random_stream_title.clone(),
            subjects: vec![random_subject],
            ..Default::default()
        })
        .await?;

    let status = client.jetstream.delete_stream(&random_stream_title).await?;
    assert!(status.success, "Stream must be deleted at this point");
    Ok(())
}

#[tokio::test]
async fn public_user_cannot_delete_stream() -> BoxedResult<()> {
    let opts = NatsClientOpts::admin_opts(NATS_URL)
        .with_rdn_namespace()
        .with_timeout(1);
    let client = NatsClient::connect(&opts).await?;

    let (random_stream_title, random_subject) =
        (gen_random_string(6), gen_random_string(6));

    client
        .jetstream
        .create_stream(types::NatsStreamConfig {
            name: random_stream_title.clone(),
            subjects: vec![random_subject],
            ..Default::default()
        })
        .await?;

    let public_opts = opts.clone().with_role(NatsUserRole::Public);
    let public_client = NatsClient::connect(&public_opts).await?;

    assert!(
        public_client
            .jetstream
            .delete_stream(&random_stream_title)
            .await
            .is_err(),
        "Stream must be deleted at this point"
    );
    Ok(())
}
