use fuel_streams_store::{
    db::{DbError, Record},
    store::{StoreError, StoreResult},
};
use fuel_streams_test::{create_random_db_name, setup_store, TestRecord};

#[tokio::test]
async fn test_add_and_retrieve_message() -> StoreResult<()> {
    let store = setup_store().await?;
    let subject = format!("{}.test.subject", create_random_db_name());
    let record = TestRecord::new("test payload");
    let db_record = store.add_record(&record, &subject).await?;

    let found_records = store.find_many_by_subject(&subject).await?;
    assert_eq!(found_records.len(), 1);
    assert_eq!(found_records[0], record);

    let deserialized_record = TestRecord::from_db_record(&db_record);
    let serialized_record = record.serialize_value();
    assert_eq!(db_record.subject, subject);
    assert_eq!(deserialized_record, record);
    assert_eq!(db_record.value, serialized_record);

    Ok(())
}

#[tokio::test]
async fn test_delete_message() -> StoreResult<()> {
    let store = setup_store().await?;
    let subject = format!("{}.test.subject", create_random_db_name());
    let record = TestRecord::new("test payload");
    store.add_record(&record, &subject).await?;

    // Verify record exists
    let records = store.find_many_by_subject(&subject).await?;
    assert_eq!(records.len(), 1);

    // Delete and verify deletion
    store.delete_record(&record, &subject).await?;
    let records = store.find_many_by_subject(&subject).await?;
    assert_eq!(records.len(), 0);

    // Test deleting non-existent record
    let err = store.delete_record(&record, &subject).await.unwrap_err();
    assert!(matches!(
        err,
        StoreError::Db(DbError::NotFound(subject)) if subject == subject
    ));

    Ok(())
}

#[tokio::test]
async fn test_update_message() -> StoreResult<()> {
    let store = setup_store().await?;
    let subject = format!("{}.test.subject", create_random_db_name());
    let record = TestRecord::new("initial payload");
    store.add_record(&record, &subject).await?;

    // Update record
    let updated_record = TestRecord::new("updated payload");
    store.update_record(&updated_record, &subject).await?;

    // Verify update
    let records = store.find_many_by_subject(&subject).await?;
    assert_eq!(records.len(), 1);
    assert_eq!(records[0], updated_record);

    // Test updating non-existent record
    let subject = format!("{}.nonexistent", create_random_db_name());
    let err = store
        .update_record(&TestRecord::new("test"), &subject)
        .await
        .unwrap_err();

    assert!(matches!(
        err,
        StoreError::Db(DbError::NotFound(subject)) if subject == subject
    ));

    Ok(())
}

#[tokio::test]
async fn test_upsert_message() -> StoreResult<()> {
    let store = setup_store().await?;
    let subject = format!("{}.test.subject", create_random_db_name());

    // Test insert
    let record = TestRecord::new("initial payload");
    store.upsert_record(&record, &subject).await?;

    let records = store.find_many_by_subject(&subject).await?;
    assert_eq!(records.len(), 1);
    assert_eq!(records[0], record);

    // Test update
    let updated_record = TestRecord::new("updated payload");
    store.upsert_record(&updated_record, &subject).await?;

    let records = store.find_many_by_subject(&subject).await?;
    assert_eq!(records.len(), 1);
    assert_eq!(records[0], updated_record);

    Ok(())
}
