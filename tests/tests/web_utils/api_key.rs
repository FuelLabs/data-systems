use std::sync::Arc;

use fuel_streams_store::db::{Db, DbConnectionOpts};
use fuel_streams_test::close_db;
use fuel_web_utils::api_key::*;
use pretty_assertions::assert_eq;
use rand::Rng;

async fn setup_test_db() -> Arc<Db> {
    let opts = DbConnectionOpts::default();
    Db::new(opts).await.expect("Failed to connect to database")
}

async fn random_user_name() -> ApiKeyUserName {
    let user_name = rand::rng().random_range(0..1000000);
    ApiKeyUserName::new(format!("user_{}", user_name))
}

#[tokio::test]
async fn test_create_and_fetch_api_key() {
    let db = setup_test_db().await;
    let pool = db.pool_ref();
    let user_name = random_user_name().await;
    let api_key = ApiKey::create(pool, &user_name, &ApiKeyRoleName::Builder)
        .await
        .expect("Failed to create API key");

    // Verify the API key was created with correct data
    assert_eq!(api_key.user(), &user_name);
    assert_eq!(api_key.role().name(), &ApiKeyRoleName::Builder);
    assert_eq!(api_key.status(), &ApiKeyStatus::Active);

    // Fetch the API key by its value
    let fetched_key = ApiKey::fetch_by_key(pool, api_key.key())
        .await
        .expect("Failed to fetch API key");

    // Verify the fetched key matches the created one
    assert_eq!(fetched_key.id(), api_key.id());
    assert_eq!(fetched_key.user(), api_key.user());
    assert_eq!(fetched_key.key(), api_key.key());
    assert_eq!(fetched_key.status(), api_key.status());
    assert_eq!(fetched_key.role().name(), api_key.role().name());

    close_db(&db).await;
}

#[tokio::test]
async fn test_update_api_key_status() {
    let db = setup_test_db().await;
    let pool = db.pool_ref();

    // Create a new API key
    let user_name = random_user_name().await;
    let api_key = ApiKey::create(pool, &user_name, &ApiKeyRoleName::WebClient)
        .await
        .expect("Failed to create API key");

    // Update the API key status to inactive
    let updated_key =
        ApiKey::update_status(pool, api_key.key(), ApiKeyStatus::Inactive)
            .await
            .expect("Failed to update API key status");

    // Verify the status was updated
    assert_eq!(updated_key.status(), &ApiKeyStatus::Inactive);

    close_db(&db).await;
}

#[tokio::test]
async fn test_api_key_role_scopes() {
    let db = setup_test_db().await;
    let pool = db.pool_ref();

    // Create API keys with different roles
    let user_admin = random_user_name().await;
    let user_web_client = random_user_name().await;

    // Admin key with FULL scope
    let admin_key = ApiKey::create(pool, &user_admin, &ApiKeyRoleName::Admin)
        .await
        .expect("Failed to create admin API key");

    // Web client key with limited scopes
    let web_client_key =
        ApiKey::create(pool, &user_web_client, &ApiKeyRoleName::WebClient)
            .await
            .expect("Failed to create web client API key");

    // Verify admin key has FULL scope
    let admin_scopes = admin_key.role().scopes();
    assert!(admin_scopes.iter().any(|s| s.is_full()));

    // Verify web client key has expected scopes
    let web_client_scopes = web_client_key.role().scopes();
    assert!(!web_client_scopes.iter().any(|s| s.is_full()));
    assert!(web_client_scopes.iter().any(|s| s.is_live_data()));
    assert!(web_client_scopes.iter().any(|s| s.is_rest_api()));

    close_db(&db).await;
}

#[tokio::test]
async fn test_fetch_all_api_keys() {
    let db = setup_test_db().await;
    let pool = db.pool_ref();

    // Create multiple API keys with different roles
    let user1 = random_user_name().await;
    let user2 = random_user_name().await;
    let user3 = random_user_name().await;

    let key1 = ApiKey::create(pool, &user1, &ApiKeyRoleName::Admin)
        .await
        .expect("Failed to create admin API key");

    let key2 = ApiKey::create(pool, &user2, &ApiKeyRoleName::Builder)
        .await
        .expect("Failed to create builder API key");

    let key3 = ApiKey::create(pool, &user3, &ApiKeyRoleName::WebClient)
        .await
        .expect("Failed to create web client API key");

    // Fetch all API keys
    let all_keys = ApiKey::fetch_all(pool)
        .await
        .expect("Failed to fetch all API keys");

    // Verify that all created keys are in the result
    // Note: There might be other keys in the database from previous tests
    let created_keys = vec![key1, key2, key3];
    for key in created_keys {
        assert!(
            all_keys.iter().any(|k| k.id() == key.id()),
            "Created key with ID {} not found in fetch_all results",
            key.id()
        );
    }

    // Verify that the keys have the correct roles
    let admin_keys: Vec<_> = all_keys
        .iter()
        .filter(|k| k.role().name() == &ApiKeyRoleName::Admin)
        .collect();
    assert!(!admin_keys.is_empty(), "No admin keys found");

    let builder_keys: Vec<_> = all_keys
        .iter()
        .filter(|k| k.role().name() == &ApiKeyRoleName::Builder)
        .collect();
    assert!(!builder_keys.is_empty(), "No builder keys found");

    let web_client_keys: Vec<_> = all_keys
        .iter()
        .filter(|k| k.role().name() == &ApiKeyRoleName::WebClient)
        .collect();
    assert!(!web_client_keys.is_empty(), "No web client keys found");

    close_db(&db).await;
}
