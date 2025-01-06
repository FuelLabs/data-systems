use fuel_streams_store::{
    storage::{CockroachStorageError, StorageError},
    store::{StoreError, StoreResult},
};
use fuel_streams_test::{
    create_random_db_name,
    create_test_record,
    setup_store,
    TestRecord,
};

#[tokio::test]
async fn test_add_and_retrieve_message() -> StoreResult<()> {
    let store = setup_store().await?;
    let key = format!("{}.test.subject", create_random_db_name());
    let record = create_test_record(&key, TestRecord::new("test payload"));
    store.add_record(&record).await?;

    let found_records = store.find_by_subject(&key).await?;
    assert_eq!(found_records.len(), 1);
    assert_eq!(found_records[0], *record.payload);

    Ok(())
}

#[tokio::test]
async fn test_delete_message() -> StoreResult<()> {
    let store = setup_store().await?;
    let key = format!("{}.test.subject", create_random_db_name());
    let record = create_test_record(&key, TestRecord::new("test payload"));
    store.add_record(&record).await?;

    // Verify record exists
    let records = store.find_by_subject(&key).await?;
    assert_eq!(records.len(), 1);

    // Delete and verify deletion
    store.delete_record(&key).await?;
    let records = store.find_by_subject(&key).await?;
    assert_eq!(records.len(), 0);

    // Test deleting non-existent record
    let err = store.delete_record(&key).await.unwrap_err();
    assert!(matches!(
        err,
        StoreError::Storage(StorageError::Cockroach(
            CockroachStorageError::NotFound(subject)
        )) if subject == key
    ));

    Ok(())
}

#[tokio::test]
async fn test_update_message() -> StoreResult<()> {
    let store = setup_store().await?;
    let key = format!("{}.test.subject", create_random_db_name());
    let record = create_test_record(&key, TestRecord::new("initial payload"));
    store.add_record(&record).await?;

    // Update record
    let updated_record =
        create_test_record(&key, TestRecord::new("updated payload"));
    store.update_record(&updated_record).await?;

    // Verify update
    let records = store.find_by_subject(&key).await?;
    assert_eq!(records.len(), 1);
    assert_eq!(records[0], *updated_record.payload);

    // Test updating non-existent record
    let key = format!("{}.nonexistent", create_random_db_name());
    let err = store
        .update_record(&create_test_record(&key, TestRecord::new("test")))
        .await
        .unwrap_err();

    assert!(matches!(
        err,
        StoreError::Storage(StorageError::Cockroach(
            CockroachStorageError::NotFound(subject)
        )) if subject == key
    ));

    Ok(())
}

#[tokio::test]
async fn test_upsert_message() -> StoreResult<()> {
    let store = setup_store().await?;
    let key = format!("{}.test.subject", create_random_db_name());

    // Test insert
    let record = create_test_record(&key, TestRecord::new("initial payload"));
    store.upsert_record(&record).await?;

    let records = store.find_by_subject(&key).await?;
    assert_eq!(records.len(), 1);
    assert_eq!(records[0], *record.payload);

    // Test update
    let updated_record =
        create_test_record(&key, TestRecord::new("updated payload"));
    store.upsert_record(&updated_record).await?;

    let records = store.find_by_subject(&key).await?;
    assert_eq!(records.len(), 1);
    assert_eq!(records[0], *updated_record.payload);

    Ok(())
}
