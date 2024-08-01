use std::collections::HashSet;

use fuel_streams::prelude::*;
use futures_util::TryStreamExt;
use streams_core::prelude::*;
use streams_tests::TestSetupBuilder;

static URL: &str = "nats://localhost:4222";

#[tokio::test]
async fn conn_streams_has_same_context_streams() -> BoxedResult<()> {
    let ctx = TestSetupBuilder::as_admin(None).await?;
    let client = ctx.conn.client();
    let streams = ConnStreams::new(&client).await?;
    let stream_list = streams.get_stream_list();
    let mut jetstreams_streams = client.context_streams();

    let mut found = HashSet::new();
    while let Some(stream) = jetstreams_streams.try_next().await? {
        found.insert(stream.config.name.clone());
    }

    for mut stream in stream_list {
        let info = stream.info().await?;
        assert!(
            found.contains(&info.config.name),
            "Stream {} not found in JetStream",
            info.config.name
        );
    }

    Ok(())
}

#[tokio::test]
async fn fuel_streams_client_connection() -> BoxedResult<()> {
    // We need to first connect as admin to have the streams created by first hand
    // since public users doesnt have permissions to
    let ctx = TestSetupBuilder::as_admin(None).await?;
    assert!(Client::new(URL)
        .with_conn_id(ctx.conn_id())
        .connect()
        .await
        .is_ok());

    Ok(())
}
