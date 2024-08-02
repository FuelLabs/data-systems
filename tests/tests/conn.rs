use std::collections::HashSet;

use futures_util::TryStreamExt;
use streams_core::prelude::*;

#[tokio::test]
async fn conn_streams_has_same_context_streams() -> BoxedResult<()> {
    let opts = ClientOpts::admin_opts(NATS_URL);
    let conn = NatsConn::connect(opts).await?;
    let stream_list = conn.streams().get_stream_list();
    let mut jetstreams_streams = conn.jetstream().streams();

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
