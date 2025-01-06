use std::time::Duration;

use fuel_streams_store::{
    db::CockroachConnectionOpts,
    store::{CacheConfig, Store, StoreResult},
};
use fuel_streams_test::{
    create_random_db_name,
    create_test_record,
    TestRecord,
};

#[tokio::test]
async fn test_cache_operations() -> StoreResult<()> {
    let config = CacheConfig {
        capacity: 100,
        ttl: Duration::from_secs(10),
        enabled: true,
    };
    let opts = CockroachConnectionOpts::default();
    let store = Store::with_cache_config(opts, config).await?;

    // Add a record
    let key = format!("{}.test.subject", create_random_db_name());
    let record = create_test_record(&key, TestRecord::new("test payload"));
    store.add_record(&record).await?;

    // First query should cache the result
    let initial_stats = store.cache_stats();
    let _ = store.find_by_subject(&key).await?;
    let after_first_query = store.cache_stats();
    assert!(after_first_query.misses >= initial_stats.misses);

    // Second query should hit the cache
    let _ = store.find_by_subject(&key).await?;
    let after_second_query = store.cache_stats();
    assert!(after_second_query.hits >= after_first_query.hits);
    assert_eq!(after_second_query.misses, after_first_query.misses);

    // Update should update the cache
    let updated_record =
        create_test_record(&key, TestRecord::new("updated payload"));
    store.update_record(&updated_record).await?;

    // Query should hit cache and get updated value
    let records = store.find_by_subject(&key).await?;
    assert_eq!(records[0], *updated_record.payload);
    let final_stats = store.cache_stats();
    assert!(final_stats.hits >= after_second_query.hits);

    // Delete should remove from cache
    store.delete_record(&key).await?;
    let records = store.find_by_subject(&key).await?;
    assert!(records.is_empty());
    let after_delete = store.cache_stats();
    assert!(after_delete.misses >= final_stats.misses);

    Ok(())
}
