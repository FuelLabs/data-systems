use async_nats::{
    jetstream::{
        kv::{self, Store},
        stream::{self, Compression, Stream},
        Context,
    },
    ConnectOptions,
};

#[allow(dead_code)]
#[derive(Clone)]
pub struct NatsHelper {
    pub client: async_nats::Client,
    pub kv_blocks: Store,
    pub kv_transactions: Store,
    pub context: Context,
    pub stream_blocks: Stream,
    pub stream_transactions: Stream,
}

impl NatsHelper {
    pub async fn connect() -> anyhow::Result<Self> {
        let client = connect().await?;
        let (
            context,
            kv_blocks,
            kv_transactions,
            stream_blocks,
            stream_transactions,
        ) = create_resources(&client).await?;
        Ok(Self {
            client,
            context,
            kv_blocks,
            kv_transactions,
            stream_blocks,
            stream_transactions,
        })
    }
}

pub async fn connect() -> anyhow::Result<async_nats::Client> {
    Ok(ConnectOptions::new()
        .user_and_password("admin".into(), "secret".into())
        .connect("localhost:4222")
        .await?)
}

async fn create_resources(
    client: &async_nats::Client,
) -> anyhow::Result<(Context, Store, Store, Stream, Stream)> {
    let jetstream = async_nats::jetstream::new(client.clone());

    // ------------------------------------------------------------------------
    // BLOCKS
    // ------------------------------------------------------------------------
    let stream_blocks = jetstream
        .get_or_create_stream(stream::Config {
            name: "blocks_encoded".into(),
            subjects: vec!["blocks.encoded.>".into()],
            compression: Some(Compression::S2),
            ..Default::default()
        })
        .await?;

    // TRANSACTIONS
    // ------------------------------------------------------------------------
    let stream_transactions = jetstream
        .get_or_create_stream(stream::Config {
            name: "transactions_encoded".into(),
            subjects: vec!["transactions.encoded.>".into()],
            compression: Some(Compression::S2),
            ..Default::default()
        })
        .await?;

    // KV STORE
    // ------------------------------------------------------------------------
    let kv_blocks = jetstream
        .create_key_value(kv::Config {
            compression: true,
            bucket: "blocks".into(),
            ..Default::default()
        })
        .await?;

    let kv_transactions = jetstream
        .create_key_value(kv::Config {
            compression: true,
            bucket: "transactions".into(),
            ..Default::default()
        })
        .await?;

    Ok((
        jetstream,
        kv_blocks,
        kv_transactions,
        stream_blocks,
        stream_transactions,
    ))
}
