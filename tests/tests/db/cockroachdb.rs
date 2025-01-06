use fuel_streams_store::db::{CockroachDbError, Db, DbError, DbResult};
use fuel_streams_test::{create_random_db_name, create_test_db};

#[tokio::test]
async fn cockroachdb_starts_and_stops() -> DbResult<()> {
    let _ = create_test_db().await?;
    Ok(())
}

#[tokio::test]
async fn cockroachdb_performs_basic_crud_operations() -> DbResult<()> {
    let db = create_test_db().await?;
    let subject = format!("{}.subject", create_random_db_name());

    // Test insert and find
    let data = b"test data".to_vec();
    let result = db.insert(&subject, &data).await?;
    assert_eq!(result.subject, subject);
    assert_eq!(result.value, data);

    let items = db.find_by_pattern(&subject).await?;
    assert_eq!(items.len(), 1);
    assert_eq!(items[0].subject, subject);
    assert_eq!(items[0].value, data);

    // Test update
    let updated_data = b"updated data".to_vec();
    let result = db.update(&subject, &updated_data).await?;
    assert_eq!(result.subject, subject);
    assert_eq!(result.value, updated_data);

    let items = db.find_by_pattern(&subject).await?;
    assert_eq!(items.len(), 1);
    assert_eq!(items[0].subject, subject);
    assert_eq!(items[0].value, updated_data);

    // Test delete
    db.delete(&subject).await?;
    let items = db.find_by_pattern(&subject).await?;
    assert!(items.is_empty());

    Ok(())
}

#[tokio::test]
async fn cockroachdb_performs_upsert_operations() -> DbResult<()> {
    let db = create_test_db().await?;
    let subject = format!("{}.subject", create_random_db_name());

    // Test insert case of upsert (new subject)
    let initial_data = b"initial data".to_vec();
    let result = db.upsert(&subject, &initial_data).await?;
    assert_eq!(result.subject, subject);
    assert_eq!(result.value, initial_data);

    // Verify the data was inserted
    let items = db.find_by_pattern(&subject).await?;
    assert_eq!(items.len(), 1);
    assert_eq!(items[0].value, initial_data);

    // Test update case of upsert (existing subject)
    let updated_data = b"updated data".to_vec();
    let result = db.upsert(&subject, &updated_data).await?;
    assert_eq!(result.subject, subject);
    assert_eq!(result.value, updated_data);

    // Verify the data was updated
    let items = db.find_by_pattern(&subject).await?;
    assert_eq!(items.len(), 1);
    assert_eq!(items[0].value, updated_data);

    Ok(())
}

#[tokio::test]
async fn cockroachdb_matches_wildcard_patterns() -> DbResult<()> {
    let db = create_test_db().await?;
    let prefix = create_random_db_name();
    let with_prefix = |subject: &str| format!("{}.{}", prefix, subject);

    // Insert test data
    let items = vec![
        (with_prefix("orders.new"), b"new order".to_vec()),
        (with_prefix("orders.processed"), b"processed order".to_vec()),
        (with_prefix("users.new"), b"new user".to_vec()),
    ];

    for (subject, data) in &items {
        db.insert(subject, data).await?;
    }

    // Test different patterns
    let orders = db.find_by_pattern(&with_prefix("orders.>")).await?;
    assert_eq!(orders.len(), 2);

    let new_items = db.find_by_pattern(&with_prefix("*.new")).await?;
    assert_eq!(new_items.len(), 2);

    let specific = db.find_by_pattern(&with_prefix("orders.new")).await?;
    assert_eq!(specific.len(), 1);
    assert_eq!(specific[0].value, b"new order");

    Ok(())
}

#[tokio::test]
async fn cockroachdb_handles_missing_items() -> DbResult<()> {
    let db = create_test_db().await?;
    let subject = format!("{}.nonexistent", create_random_db_name());

    // Test updating non-existent item
    let data = b"test".to_vec();
    let err = db.update(&subject, &data).await.unwrap_err();
    match err {
        DbError::Cockroach(CockroachDbError::NotFound(subject)) => {
            assert_eq!(subject, subject);
        }
        _ => panic!("Expected NotFound error"),
    }

    // Test deleting non-existent item
    let err = db.delete(&subject).await.unwrap_err();
    match err {
        DbError::Cockroach(CockroachDbError::NotFound(subject)) => {
            assert_eq!(subject, subject);
        }
        _ => panic!("Expected NotFound error"),
    }

    Ok(())
}

#[tokio::test]
async fn cockroachdb_handles_cleanup_tables() -> DbResult<()> {
    let _ = create_test_db().await?;
    Ok(())
}

#[tokio::test]
async fn cockroachdb_fails_duplicate_subjects() {
    let db = create_test_db().await.unwrap();
    let subject = format!("{}.duplicate.subject", create_random_db_name());
    db.insert(&subject, b"duplicate data").await.unwrap();
    let res = db.insert(&subject, b"duplicate data").await;
    assert!(res.is_err());
}
