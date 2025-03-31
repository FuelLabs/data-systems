use std::sync::Arc;

use fuel_streams_domains::infra::{Db, DbConnectionOpts};
use fuel_streams_test::close_db;
use fuel_streams_types::BlockHeight;
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
    assert!(roles.iter().any(|r| r.name().is_amm()));
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
    assert!(admin_role.scopes().iter().any(|s| s.is_manage_api_keys()));
    assert!(admin_role.scopes().iter().any(|s| s.is_historical_data()));
    assert!(admin_role.scopes().iter().any(|s| s.is_live_data()));
    assert!(admin_role.scopes().iter().any(|s| s.is_rest_api()));
    assert_eq!(admin_role.scopes().len(), 4);

    let web_client_role =
        ApiKeyRole::fetch_by_name(pool, &ApiKeyRoleName::WebClient)
            .await
            .expect("Failed to fetch web client role");
    assert!(!web_client_role
        .scopes()
        .iter()
        .any(|s| s.is_manage_api_keys()));
    assert!(web_client_role.scopes().iter().any(|s| s.is_live_data()));
    assert!(web_client_role.scopes().iter().any(|s| s.is_rest_api()));
    assert_eq!(web_client_role.scopes().len(), 2);

    let amm_role = ApiKeyRole::fetch_by_name(pool, &ApiKeyRoleName::Amm)
        .await
        .expect("Failed to fetch amm role");
    assert!(amm_role.scopes().iter().any(|s| s.is_historical_data()));
    assert!(amm_role.scopes().iter().any(|s| s.is_live_data()));
    assert!(amm_role.scopes().iter().any(|s| s.is_rest_api()));
    assert_eq!(amm_role.scopes().len(), 3);

    let builder_role =
        ApiKeyRole::fetch_by_name(pool, &ApiKeyRoleName::Builder)
            .await
            .expect("Failed to fetch builder role");
    assert!(builder_role.scopes().iter().any(|s| s.is_historical_data()));
    assert!(builder_role.scopes().iter().any(|s| s.is_live_data()));
    assert!(builder_role.scopes().iter().any(|s| s.is_rest_api()));
    assert_eq!(builder_role.scopes().len(), 3);
    close_db(&db).await;
}

#[tokio::test]
async fn test_fetch_role_by_id() {
    let db = setup_test_db().await;
    let pool = db.pool_ref();
    let admin_role = ApiKeyRole::fetch_by_name(pool, &ApiKeyRoleName::Admin)
        .await
        .expect("Failed to fetch admin role");

    let role = ApiKeyRole::fetch_by_id(pool, *admin_role.id())
        .await
        .expect("Failed to fetch role by ID");
    assert_eq!(role.name(), &ApiKeyRoleName::Admin);
    close_db(&db).await;
}

#[tokio::test]
async fn test_validate_historical_days_limit() -> anyhow::Result<()> {
    let db = setup_test_db().await;
    let pool = db.pool_ref();

    // Get the builder role which has a 7-day historical limit
    let builder_role =
        ApiKeyRole::fetch_by_name(pool, &ApiKeyRoleName::Builder)
            .await
            .expect("Failed to fetch builder role");

    let last_height = BlockHeight::from(700);
    let within_limit = BlockHeight::from(650);
    assert!(builder_role
        .validate_historical_limit(last_height, within_limit)
        .is_ok());

    let beyond_limit = BlockHeight::from(99);
    assert!(builder_role
        .validate_historical_limit(last_height, beyond_limit)
        .is_err());

    // Test with role that has no historical limit (should always pass)
    let admin_role = ApiKeyRole::fetch_by_name(pool, &ApiKeyRoleName::Admin)
        .await
        .expect("Failed to fetch admin role");

    let very_old = BlockHeight::from(1);
    assert!(admin_role
        .validate_historical_limit(last_height, very_old)
        .is_ok());

    close_db(&db).await;
    Ok(())
}
