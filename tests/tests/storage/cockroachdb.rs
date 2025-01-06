use fuel_streams_store::storage::{
    CockroachStorageError,
    Storage,
    StorageError,
    StorageResult,
};
use fuel_streams_test::{create_random_db_name, create_test_storage};

#[tokio::test]
async fn cockroachdb_starts_and_stops() -> StorageResult<()> {
    let _ = create_test_storage().await?;
    Ok(())
}

#[tokio::test]
async fn cockroachdb_performs_basic_crud_operations() -> StorageResult<()> {
    let storage = create_test_storage().await?;
    let subject = format!("{}.subject", create_random_db_name());

    // Test insert and find
    let data = b"test data".to_vec();
    let result = storage.insert(&subject, &data).await?;
    assert_eq!(result.subject, subject);
    assert_eq!(result.value, data);

    let items = storage.find_by_pattern(&subject).await?;
    assert_eq!(items.len(), 1);
    assert_eq!(items[0].subject, subject);
    assert_eq!(items[0].value, data);

    // Test update
    let updated_data = b"updated data".to_vec();
    let result = storage.update(&subject, &updated_data).await?;
    assert_eq!(result.subject, subject);
    assert_eq!(result.value, updated_data);

    let items = storage.find_by_pattern(&subject).await?;
    assert_eq!(items.len(), 1);
    assert_eq!(items[0].subject, subject);
    assert_eq!(items[0].value, updated_data);

    // Test delete
    storage.delete(&subject).await?;
    let items = storage.find_by_pattern(&subject).await?;
    assert!(items.is_empty());

    Ok(())
}

#[tokio::test]
async fn cockroachdb_performs_upsert_operations() -> StorageResult<()> {
    let storage = create_test_storage().await?;
    let subject = format!("{}.subject", create_random_db_name());

    // Test insert case of upsert (new subject)
    let initial_data = b"initial data".to_vec();
    let result = storage.upsert(&subject, &initial_data).await?;
    assert_eq!(result.subject, subject);
    assert_eq!(result.value, initial_data);

    // Verify the data was inserted
    let items = storage.find_by_pattern(&subject).await?;
    assert_eq!(items.len(), 1);
    assert_eq!(items[0].value, initial_data);

    // Test update case of upsert (existing subject)
    let updated_data = b"updated data".to_vec();
    let result = storage.upsert(&subject, &updated_data).await?;
    assert_eq!(result.subject, subject);
    assert_eq!(result.value, updated_data);

    // Verify the data was updated
    let items = storage.find_by_pattern(&subject).await?;
    assert_eq!(items.len(), 1);
    assert_eq!(items[0].value, updated_data);

    Ok(())
}

#[tokio::test]
async fn cockroachdb_matches_wildcard_patterns() -> StorageResult<()> {
    let storage = create_test_storage().await?;
    let prefix = create_random_db_name();
    let with_prefix = |subject: &str| format!("{}.{}", prefix, subject);

    // Insert test data
    let items = vec![
        (with_prefix("orders.new"), b"new order".to_vec()),
        (with_prefix("orders.processed"), b"processed order".to_vec()),
        (with_prefix("users.new"), b"new user".to_vec()),
    ];

    for (subject, data) in &items {
        storage.insert(subject, data).await?;
    }

    // Test different patterns
    let orders = storage.find_by_pattern(&with_prefix("orders.>")).await?;
    assert_eq!(orders.len(), 2);

    let new_items = storage.find_by_pattern(&with_prefix("*.new")).await?;
    assert_eq!(new_items.len(), 2);

    let specific = storage.find_by_pattern(&with_prefix("orders.new")).await?;
    assert_eq!(specific.len(), 1);
    assert_eq!(specific[0].value, b"new order");

    Ok(())
}

#[tokio::test]
async fn cockroachdb_handles_missing_items() -> StorageResult<()> {
    let storage = create_test_storage().await?;
    let subject = format!("{}.nonexistent", create_random_db_name());

    // Test updating non-existent item
    let data = b"test".to_vec();
    let err = storage.update(&subject, &data).await.unwrap_err();
    match err {
        StorageError::Cockroach(CockroachStorageError::NotFound(subject)) => {
            assert_eq!(subject, subject);
        }
        _ => panic!("Expected NotFound error"),
    }

    // Test deleting non-existent item
    let err = storage.delete(&subject).await.unwrap_err();
    match err {
        StorageError::Cockroach(CockroachStorageError::NotFound(subject)) => {
            assert_eq!(subject, subject);
        }
        _ => panic!("Expected NotFound error"),
    }

    Ok(())
}

#[tokio::test]
async fn cockroachdb_handles_cleanup_tables() -> StorageResult<()> {
    let _ = create_test_storage().await?;
    Ok(())
}

#[tokio::test]
async fn cockroachdb_fails_duplicate_subjects() {
    let storage = create_test_storage().await.unwrap();
    let subject = format!("{}.duplicate.subject", create_random_db_name());
    storage.insert(&subject, b"duplicate data").await.unwrap();
    let res = storage.insert(&subject, b"duplicate data").await;
    assert!(res.is_err());
}
