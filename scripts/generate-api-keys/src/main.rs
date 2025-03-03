use std::sync::Arc;

use fuel_streams_store::db::{Db, DbConnectionOpts};
use fuel_web_utils::api_key::{ApiKey, ApiKeyRoleName};
use generate_api_keys::config::Config;
use sqlx::{Postgres, Transaction};
use strum::IntoEnumIterator;
use tracing::level_filters::LevelFilter;
use tracing_subscriber::{fmt::format::FmtSpan, EnvFilter};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    setup_tracing();
    load_env_file();

    let config = Config::load()?;
    let db = connect_to_database(&config).await?;
    let roles = ApiKeyRoleName::iter().collect::<Vec<_>>();
    let keys_per_role = roles.len();

    tracing::info!(
        "Generating {} API keys for each role ({} total)",
        keys_per_role,
        keys_per_role * roles.len()
    );

    let mut tx = db.pool.begin().await?;

    // Add special test key
    add_special_test_key(&mut tx).await?;

    // Generate regular keys for each role
    generate_keys_for_roles(&mut tx, &roles, keys_per_role).await?;

    if let Err(e) = tx.commit().await {
        tracing::error!("Failed to commit transaction: {:?}", e);
    }

    tracing::info!(
        "Generated {} API keys and stored into db",
        keys_per_role * roles.len() + 1 // +1 for the special test key
    );
    Ok(())
}

fn setup_tracing() {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::builder()
                .with_default_directive(LevelFilter::INFO.into())
                .from_env_lossy(),
        )
        .with_span_events(FmtSpan::CLOSE)
        .init();
}

fn load_env_file() {
    if let Err(err) = dotenvy::dotenv() {
        tracing::warn!("File .env not found: {:?}", err);
    }
}

async fn connect_to_database(config: &Config) -> anyhow::Result<Arc<Db>> {
    let db = Db::new(DbConnectionOpts {
        connection_str: config.db.url.clone(),
        ..Default::default()
    })
    .await?;
    Ok(db)
}

async fn add_special_test_key(
    tx: &mut Transaction<'_, Postgres>,
) -> anyhow::Result<()> {
    let admin_role_id = get_role_id(tx, &ApiKeyRoleName::Admin).await?;
    tracing::info!("Adding special test key with ADMIN role");
    sqlx::query(
        "INSERT INTO api_keys (user_name, api_key, role_id)
        VALUES ($1, $2, $3)
        RETURNING id, user_name, api_key",
    )
    .bind("test")
    .bind("your_key")
    .bind(admin_role_id)
    .fetch_one(&mut **tx)
    .await?;

    Ok(())
}

async fn generate_keys_for_roles(
    tx: &mut Transaction<'_, Postgres>,
    roles: &[ApiKeyRoleName],
    keys_per_role: usize,
) -> anyhow::Result<()> {
    for role in roles.iter() {
        let role_id = get_role_id(tx, role).await?;
        tracing::info!("Generating {} keys for role: {}", keys_per_role, role);

        for i in 0..keys_per_role {
            let user_name = format!("{}-{}", role, i + 1);
            tracing::info!("Generated new db record for {}", user_name);
            insert_api_key(tx, &user_name, role_id).await?;
        }
    }

    Ok(())
}

async fn get_role_id(
    tx: &mut Transaction<'_, Postgres>,
    role_name: &ApiKeyRoleName,
) -> anyhow::Result<i32> {
    let role_id: i32 = sqlx::query_scalar(
        "SELECT id FROM api_key_roles WHERE name = $1::api_role",
    )
    .bind(role_name)
    .fetch_one(&mut **tx)
    .await?;

    Ok(role_id)
}

async fn insert_api_key(
    tx: &mut Transaction<'_, Postgres>,
    user_name: &str,
    role_id: i32,
) -> anyhow::Result<()> {
    sqlx::query(
        "INSERT INTO api_keys (user_name, api_key, role_id)
        VALUES ($1, $2, $3)
        RETURNING id, user_name, api_key",
    )
    .bind(user_name)
    .bind(ApiKey::generate_random_api_key())
    .bind(role_id)
    .fetch_one(tx.as_mut())
    .await?;
    Ok(())
}
