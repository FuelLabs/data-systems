use async_nats::connection::State;
use fuel_streams::prelude::*;
use futures_util::future::try_join_all;
use pretty_assertions::assert_eq;
use streams_core::prelude::*;

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
