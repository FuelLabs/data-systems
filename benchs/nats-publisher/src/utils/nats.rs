use async_nats::{
    jetstream::{
        kv::{self, Store},
        stream::{self, Compression, Stream},
        Context,
    },
    ConnectOptions,
};
use fuel_data_parser::{DataParser, DataParserBuilder};

#[allow(dead_code)]
#[derive(Clone)]
pub struct NatsHelper {
    pub client: async_nats::Client,
    pub kv_blocks: Store,
    pub kv_transactions: Store,
    pub context: Context,
    pub stream_blocks: Stream,
    pub stream_transactions: Stream,
    pub use_nats_compression: bool,
    pub data_parser: DataParser,
}

impl NatsHelper {
    pub async fn connect(use_nats_compression: bool) -> anyhow::Result<Self> {
        let client = connect().await?;
        let (
            context,
            kv_blocks,
            kv_transactions,
            stream_blocks,
            stream_transactions,
        ) = create_resources(&client, use_nats_compression).await?;
        let data_parser = DataParserBuilder::default().build();
        Ok(Self {
            client,
            context,
            kv_blocks,
            kv_transactions,
            stream_blocks,
            stream_transactions,
            use_nats_compression,
            data_parser,
        })
    }

    #[allow(dead_code)]
    pub fn data_parser(&self) -> &DataParser {
        &self.data_parser
    }

    #[allow(dead_code)]
    pub fn data_parser_mut(&mut self) -> &mut DataParser {
        &mut self.data_parser
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
    use_nats_compression: bool,
) -> anyhow::Result<(Context, Store, Store, Stream, Stream)> {
    let jetstream = async_nats::jetstream::new(client.clone());

    // ------------------------------------------------------------------------
    // BLOCKS
    // ------------------------------------------------------------------------
    let stream_blocks = jetstream
        .get_or_create_stream(stream::Config {
            name: "blocks_encoded".into(),
            subjects: vec!["blocks.>".into()],
            compression: if use_nats_compression {
                Some(Compression::S2)
            } else {
                None
            },
            ..Default::default()
        })
        .await?;

    // TRANSACTIONS
    // ------------------------------------------------------------------------
    let stream_transactions = jetstream
        .get_or_create_stream(stream::Config {
            name: "transactions_encoded".into(),
            subjects: vec!["transactions.>".into()],
            compression: if use_nats_compression {
                Some(Compression::S2)
            } else {
                None
            },
            ..Default::default()
        })
        .await?;

    // KV STORE
    // ------------------------------------------------------------------------
    let kv_blocks = jetstream
        .create_key_value(kv::Config {
            compression: use_nats_compression,
            bucket: "blocks".into(),
            ..Default::default()
        })
        .await?;

    let kv_transactions = jetstream
        .create_key_value(kv::Config {
            compression: use_nats_compression,
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
