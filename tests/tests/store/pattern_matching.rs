use fuel_streams_store::{
    store::{StoreError, StoreResult},
    subject_validator::SubjectPatternError,
};
use fuel_streams_test::{
    add_test_records,
    create_random_db_name,
    setup_store,
    TestRecord,
};

#[tokio::test]
async fn test_asterisk_wildcards() -> StoreResult<()> {
    let store = setup_store().await?;
    let prefix = create_random_db_name();
    let with_prefix = |s: &str| format!("{}.{}", prefix, s);

    add_test_records(&store, &prefix, &[
        ("orders.new", TestRecord::new("Order")),
        ("orders.new.1", TestRecord::new("Order 1")),
        ("orders.new.2", TestRecord::new("Order 2")),
        ("products.new", TestRecord::new("Product")),
        ("products.new.1", TestRecord::new("Product 1")),
        ("orders.cancelled", TestRecord::new("Cancelled Order")),
        ("orders.cancelled.1", TestRecord::new("Cancelled Order 1")),
    ])
    .await?;

    // Test single level wildcard
    let orders = store.find_many_by_subject(&with_prefix("orders.*")).await?;
    assert_eq!(orders.len(), 2);

    // Test multi-level wildcard
    let new_things =
        store.find_many_by_subject(&with_prefix("*.new.*")).await?;
    assert_eq!(new_things.len(), 3);

    // Test wildcard at start
    let all_new = store.find_many_by_subject(&with_prefix("*.new")).await?;
    assert_eq!(all_new.len(), 2);

    // Test multiple wildcards
    let all_ones = store.find_many_by_subject(&with_prefix("*.*.1")).await?;
    assert_eq!(all_ones.len(), 3);

    // Test no matches
    let no_matches =
        store.find_many_by_subject(&with_prefix("*.old.*")).await?;
    assert_eq!(no_matches.len(), 0);

    Ok(())
}

#[tokio::test]
async fn test_greater_than_wildcards() -> StoreResult<()> {
    let store = setup_store().await?;
    let prefix = create_random_db_name();
    let with_prefix = |s: &str| format!("{}.{}", prefix, s);

    add_test_records(&store, &prefix, &[
        ("orders.processed.1", TestRecord::new("Processed Order 1")),
        ("orders.processed.2", TestRecord::new("Processed Order 2")),
        (
            "orders.processed.3.status",
            TestRecord::new("Processed Order 3 Status"),
        ),
        (
            "orders.processed.3.details",
            TestRecord::new("Processed Order 3 Details"),
        ),
        ("orders.new.1", TestRecord::new("New Order 1")),
        ("orders.new.2", TestRecord::new("New Order 2")),
        ("orders.cancelled.1", TestRecord::new("Cancelled Order 1")),
        ("products.new.1", TestRecord::new("New Product 1")),
    ])
    .await?;

    let processed_orders = store
        .find_many_by_subject(&with_prefix("orders.processed.>"))
        .await?;
    assert_eq!(processed_orders.len(), 4);

    let new_orders = store
        .find_many_by_subject(&with_prefix("orders.new.*"))
        .await?;
    assert_eq!(new_orders.len(), 2);

    let all_orders =
        store.find_many_by_subject(&with_prefix("orders.>")).await?;
    assert_eq!(all_orders.len(), 7);

    Ok(())
}

#[tokio::test]
async fn test_empty_pattern() -> StoreResult<()> {
    let store = setup_store().await?;
    let prefix = create_random_db_name();
    let with_prefix = |s: &str| format!("{}.{}", prefix, s);

    store
        .add_record(&TestRecord::new("Order 1"), &with_prefix("orders.new.1"))
        .await?;

    let err = store.find_many_by_subject("").await.unwrap_err();
    assert!(matches!(
        err,
        StoreError::InvalidSubject {
            pattern,
            error: SubjectPatternError::Empty
        } if pattern.is_empty()
    ));

    Ok(())
}

#[tokio::test]
async fn test_nonexistent_subjects() -> StoreResult<()> {
    let store = setup_store::<TestRecord>().await?;
    let prefix = create_random_db_name();
    let with_prefix = |s: &str| format!("{}.{}", prefix, s);

    let found_messages = store
        .find_many_by_subject(&with_prefix("nonexistent.subject"))
        .await?;
    assert!(found_messages.is_empty());

    Ok(())
}
