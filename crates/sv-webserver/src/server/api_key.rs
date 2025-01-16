use std::sync::Arc;

use fuel_streams_store::db::Db;
use fuel_web_utils::server::middlewares::api_key::{
    ApiKey,
    ApiKeyLimit,
    ApiKeyLimits,
    ApiKeyRestrictions,
    ApiKeyStatus,
};
use serde::{Deserialize, Serialize};

const API_KEYS_TABLE: &str = "api_keys";
const USER_ACTIVE_KEY_PREFIX: &str = "USER_ACTIVE_KEY";

fn create_deteministic_uuid(input: &str) -> uuid::Uuid {
    use sha2::{Digest, Sha256};
    let hash = Sha256::digest(input.as_bytes());
    let mut bytes = [0u8; 16];
    bytes.copy_from_slice(&hash[..16]);
    uuid::Uuid::from_bytes(bytes)
}

fn generate_random_string() -> String {
    use rand::{distributions::Alphanumeric, thread_rng, Rng};
    thread_rng()
        .sample_iter(&Alphanumeric)
        .take(30)
        .map(char::from)
        .collect()
}

#[derive(sqlx::FromRow, Debug, Clone, Serialize, Deserialize)]
pub struct DbUserApiKey {
    pub id: uuid::Uuid,
    pub api_key: String,
}

#[derive(Clone)]
pub struct ApiKeysManager {
    pub db: Arc<Db>,
}

impl ApiKeysManager {
    pub fn new(db: Arc<Db>) -> Self {
        Self { db }
    }

    pub async fn load_from_db(&self) -> anyhow::Result<Vec<ApiKey>> {
        let mut query_builder = sqlx::QueryBuilder::new(format!(
            "SELECT * FROM {}",
            API_KEYS_TABLE
        ));

        let query = query_builder.build_query_as::<DbUserApiKey>();
        let db_records = query.fetch_all(&self.db.pool).await?;

        let keys = db_records
            .into_iter()
            .map(|record| ApiKey {
                user_id: record.id,
                key: record.api_key,
                limits: ApiKeyLimits {
                    max_reads_per_minute: ApiKeyLimit::Limited(100),
                    max_writes_per_minute: ApiKeyLimit::Limited(100),
                },
                restrictions: ApiKeyRestrictions {
                    allowed_domains: vec![],
                },
                status: ApiKeyStatus::Active,
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
            })
            .collect::<Vec<ApiKey>>();

        Ok(keys)
    }

    pub fn generate_random(size: usize) -> Vec<ApiKey> {
        (0..size)
            .map(|index| ApiKey {
                user_id: create_deteministic_uuid(
                    format!("{}_{}", USER_ACTIVE_KEY_PREFIX, index).as_str(),
                ),
                key: generate_random_string(),
                limits: ApiKeyLimits {
                    max_reads_per_minute: ApiKeyLimit::Limited(100),
                    max_writes_per_minute: ApiKeyLimit::Limited(100),
                },
                restrictions: ApiKeyRestrictions {
                    allowed_domains: vec![],
                },
                status: ApiKeyStatus::Active,
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
            })
            .collect::<Vec<_>>()
    }
}
