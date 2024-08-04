use async_nats::{
    jetstream::{
        kv::{self, Store},
        stream::{self, Compression, Stream},
        Context,
    },
    ConnectOptions,
};

#[derive(Clone)]
pub struct NatsHelper {
    pub kv_blocks: Store,
    pub kv_transactions: Store,
    pub context: Context,
    pub st_blocks_encoded: Stream,
    pub st_blocks_json: Stream,
    pub st_transactions_encoded: Stream,
    pub st_transactions_json: Stream,
}

impl NatsHelper {
    pub async fn connect() -> anyhow::Result<Self> {
        let client = connect().await?;
        let (
            context,
            kv_blocks,
            kv_transactions,
            st_blocks_encoded,
            st_blocks_json,
            st_transactions_encoded,
            st_transactions_json,
        ) = create_resources(&client).await?;
        Ok(Self {
            context,
            kv_blocks,
            kv_transactions,
            st_blocks_encoded,
            st_blocks_json,
            st_transactions_encoded,
            st_transactions_json,
        })
    }
}

async fn connect() -> anyhow::Result<async_nats::Client> {
    Ok(ConnectOptions::new()
        .user_and_password("admin".into(), "secret".into())
        .connect("localhost:4222")
        .await?)
}

async fn create_resources(
    client: &async_nats::Client,
) -> anyhow::Result<(Context, Store, Store, Stream, Stream, Stream, Stream)> {
    let jetstream = async_nats::jetstream::new(client.clone());

    // ------------------------------------------------------------------------
    // BLOCKS
    // ------------------------------------------------------------------------
    let st_blocks_encoded = jetstream
        .get_or_create_stream(stream::Config {
            name: "blocks_encoded".into(),
            subjects: vec!["blocks.encoded.>".into()],
            compression: Some(Compression::S2),
            ..Default::default()
        })
        .await?;

    let st_blocks_json = jetstream
        .get_or_create_stream(stream::Config {
            name: "blocks_json".into(),
            subjects: vec!["blocks.json.>".into()],
            ..Default::default()
        })
        .await?;

    // ------------------------------------------------------------------------
    // TRANSACTIONS
    // ------------------------------------------------------------------------
    let st_transactions_encoded = jetstream
        .get_or_create_stream(stream::Config {
            name: "transactions_encoded".into(),
            subjects: vec!["transactions.encoded.>".into()],
            compression: Some(Compression::S2),
            ..Default::default()
        })
        .await?;

    let st_transactions_json = jetstream
        .get_or_create_stream(stream::Config {
            name: "transactions_json".into(),
            subjects: vec!["transactions.json.>".into()],
            ..Default::default()
        })
        .await?;

    // ------------------------------------------------------------------------
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
        st_blocks_encoded,
        st_blocks_json,
        st_transactions_encoded,
        st_transactions_json,
    ))
}
