use fuel_streams_store::db::{Db, DbConnectionOpts};
use fuel_web_utils::server::middlewares::api_key::{ApiKey, DbUserApiKey};
use generate_api_keys::config::Config;
use sqlx::{Postgres, Transaction};
use tracing::level_filters::LevelFilter;
use tracing_subscriber::{fmt::format::FmtSpan, EnvFilter};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // init tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::builder()
                .with_default_directive(LevelFilter::INFO.into())
                .from_env_lossy(),
        )
        .with_span_events(FmtSpan::CLOSE)
        .init();

    if let Err(err) = dotenvy::dotenv() {
        tracing::warn!("File .env not found: {:?}", err);
    }

    let config = Config::load()?;
    let db = Db::new(DbConnectionOpts {
        connection_str: config.db.url.clone(),
        ..Default::default()
    })
    .await?;

    tracing::info!("Generating {:?} api keys", config.api_keys.nsize);
    let user_ids = (0..config.api_keys.nsize).collect::<Vec<i32>>();
    let mut tx = db.pool.begin().await?;
    for (index, _) in user_ids.iter().enumerate() {
        tracing::info!(
            "Generated new db record {:?}",
            insert_api_key(&mut tx, index).await?
        );
    }
    if let Err(e) = tx.commit().await {
        tracing::error!("Failed to commit transaction: {:?}", e);
    }
    tracing::info!(
        "Generated {:?} api keys and stored into db",
        config.api_keys.nsize
    );
    Ok(())
}

async fn insert_api_key(
    tx: &mut Transaction<'_, Postgres>,
    index: usize,
) -> anyhow::Result<DbUserApiKey> {
    let db_record = sqlx::query_as::<_, DbUserApiKey>(
        "INSERT INTO api_keys (user_name, api_key)
        VALUES ($1, $2)
        RETURNING user_id, user_name, api_key",
    )
    .bind(format!("fuel-{}", index + 1))
    .bind(ApiKey::generate_random_api_key())
    .fetch_one(tx.as_mut())
    .await?;
    Ok(db_record)
}
