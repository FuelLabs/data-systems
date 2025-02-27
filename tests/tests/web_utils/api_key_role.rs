use std::sync::Arc;

use fuel_streams_store::db::{Db, DbConnectionOpts};
use fuel_streams_test::close_db;
use fuel_web_utils::api_key::*;
use pretty_assertions::assert_eq;

async fn setup_test_db() -> Arc<Db> {
    let opts = DbConnectionOpts::default();
    Db::new(opts).await.expect("Failed to connect to database")
}

#[tokio::test]
async fn test_fetch_all_roles() {
    let db = setup_test_db().await;
    let pool = db.pool_ref();
    let roles = ApiKeyRole::fetch_all(pool)
        .await
        .expect("Failed to fetch roles");

    assert!(!roles.is_empty());
    assert!(roles.iter().any(|r| r.name().is_admin()));
    assert!(roles.iter().any(|r| r.name().is_builder()));
    assert!(roles.iter().any(|r| r.name().is_web_client()));
    close_db(&db).await;
}

#[tokio::test]
async fn test_fetch_role_by_name() {
    let db = setup_test_db().await;
    let pool = db.pool_ref();
    let role = ApiKeyRole::fetch_by_name(pool, &ApiKeyRoleName::Admin)
        .await
        .expect("Failed to fetch role");

    assert_eq!(role.name(), &ApiKeyRoleName::Admin);
    assert!(role.scopes().iter().any(|s| s.is_full()));
    close_db(&db).await;
}

#[tokio::test]
async fn test_role_limits() {
    let db = setup_test_db().await;
    let pool = db.pool_ref();
    let builder_role =
        ApiKeyRole::fetch_by_name(pool, &ApiKeyRoleName::Builder)
            .await
            .expect("Failed to fetch builder role");

    let subscription_limit = builder_role.subscription_limit();
    assert!(
        subscription_limit.is_some(),
        "Builder role should have a subscription limit"
    );

    println!("Builder subscription limit: {:?}", subscription_limit);
    println!(
        "Builder rate limit: {:?}",
        builder_role.rate_limit_per_minute()
    );
    close_db(&db).await;
}

#[tokio::test]
async fn test_role_scopes() {
    let db = setup_test_db().await;
    let pool = db.pool_ref();

    let admin_role = ApiKeyRole::fetch_by_name(pool, &ApiKeyRoleName::Admin)
        .await
        .expect("Failed to fetch admin role");
    assert!(admin_role.scopes().iter().any(|s| s.is_full()));
    assert_eq!(admin_role.scopes().len(), 1);

    let web_client_role =
        ApiKeyRole::fetch_by_name(pool, &ApiKeyRoleName::WebClient)
            .await
            .expect("Failed to fetch web client role");
    assert!(!web_client_role.scopes().iter().any(|s| s.is_full()));
    assert!(web_client_role.scopes().iter().any(|s| s.is_live_data()));
    assert!(web_client_role.scopes().iter().any(|s| s.is_rest_api()));
    assert!(!web_client_role
        .scopes()
        .iter()
        .any(|s| s.is_historical_data()));
    assert_eq!(web_client_role.scopes().len(), 2);
    close_db(&db).await;
}

#[tokio::test]
async fn test_fetch_role_by_id() {
    let db = setup_test_db().await;
    let pool = db.pool_ref();

    let builder_role =
        ApiKeyRole::fetch_by_name(pool, &ApiKeyRoleName::Builder)
            .await
            .expect("Failed to fetch builder role");

    let role = ApiKeyRole::fetch_by_id(pool, *builder_role.id())
        .await
        .expect("Failed to fetch role by ID");

    assert_eq!(role.name(), &ApiKeyRoleName::Builder);
    assert!(role.scopes().iter().any(|s| s.is_full()));
    assert!(role.subscription_limit().is_some());
    close_db(&db).await;
}
