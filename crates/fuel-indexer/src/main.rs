use std::sync::Arc;

use fuel_streams_core::{
    nats::{types::DeliverPolicy, NatsClient, NatsClientOpts},
    s3::{S3Client, S3ClientOpts},
    types::{Block, Transaction},
    Streamable,
    SubscriptionConfig,
};
use futures::StreamExt;
use serde::{Deserialize, Serialize};
use surrealdb::{
    engine::remote::ws::{Client, Ws},
    opt::auth::Root,
    sql::Thing,
    Surreal,
};

#[derive(Debug, Serialize, Deserialize)]
struct BlockRecord {
    id: Thing,
    data: Block,
}

#[derive(Debug, Serialize, Deserialize)]
struct TransactionRecord {
    id: Thing,
    data: Transaction,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let db_url =
        dotenvy::var("SURREALDB_URL").expect("`SURREALDB_URL` env must be set");
    let db_user = dotenvy::var("SURREALDB_USER")
        .expect("`SURREALDB_USER` env must be set");
    let db_pass = dotenvy::var("SURREALDB_PASS")
        .expect("`SURREALDB_PASS` env must be set");

    let db = Surreal::new::<Ws>(db_url).await?;
    db.signin(Root {
        username: db_user.as_str(),
        password: db_pass.as_str(),
    })
    .await?;

    db.use_ns("fuel_indexer").use_db("fuel_indexer").await?;

    let nats_client_opts = NatsClientOpts::admin_opts();
    let nats_client = NatsClient::connect(&nats_client_opts).await?;

    let s3_client_opts = S3ClientOpts::admin_opts();
    let s3_client = S3Client::new(&s3_client_opts).await?.arc();

    tokio::try_join!(
        sync_blocks(&db, &nats_client, s3_client.clone()),
        sync_transactions(&db, &nats_client, s3_client)
    )?;

    Ok(())
}

async fn sync_blocks(
    db: &Surreal<Client>,
    client: &NatsClient,
    s3_client: Arc<S3Client>,
) -> anyhow::Result<()> {
    let stream =
        fuel_streams_core::Stream::<Block>::get_or_init(client, &s3_client)
            .await;

    let mut subscription = stream
        .subscribe(Some(SubscriptionConfig {
            deliver_policy: DeliverPolicy::All,
            filter_subjects: vec![Block::WILDCARD_LIST[0].to_string()],
        }))
        .await?;

    while let Some(block) = subscription.next().await {
        let height = block.height;
        let id = height.to_string();
        let key = ("block".to_string(), id.clone());
        let record: Option<BlockRecord> = db
            .upsert(key.clone())
            .content(BlockRecord {
                id: key.into(),
                data: block,
            })
            .await?;

        dbg!(record);
    }
    Ok(())
}

async fn sync_transactions(
    db: &Surreal<Client>,
    client: &NatsClient,
    s3_client: Arc<S3Client>,
) -> anyhow::Result<()> {
    let stream = fuel_streams_core::Stream::<Transaction>::get_or_init(
        client, &s3_client,
    )
    .await;

    let mut subscription = stream
        .subscribe(Some(SubscriptionConfig {
            deliver_policy: DeliverPolicy::All,
            filter_subjects: vec![Transaction::WILDCARD_LIST[0].to_string()],
        }))
        .await?;

    while let Some(transaction) = subscription.next().await {
        let tx_id = &transaction.id;
        let id = format!("0x{}", tx_id);
        let key = ("transaction".to_string(), id.clone());
        let record: Option<TransactionRecord> = db
            .upsert(key.clone())
            .content(TransactionRecord {
                id: key.into(),
                data: transaction,
            })
            .await?;

        dbg!(record);
    }

    Ok(())
}
