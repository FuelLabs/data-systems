use std::time::Duration;

use fuel_streams_store::{
    db::DbConnectionOpts,
    store::{CacheConfig, Store, StoreResult},
};
use fuel_streams_test::{create_random_db_name, TestRecord};

#[tokio::test]
async fn test_cache_operations() -> StoreResult<()> {
    let config = CacheConfig {
        capacity: 100,
        ttl: Duration::from_secs(10),
        enabled: true,
    };
    let opts = DbConnectionOpts::default();
    let store = Store::<TestRecord>::with_cache_config(opts, config).await?;

    // Add a record
    let subject = format!("{}.test.subject", create_random_db_name());
    let record = TestRecord::new("test payload");
    let packet = record.to_packet(&subject);
    store.add_record(&packet).await?;

    // First query should cache the result
    let initial_stats = store.cache_stats();
    let _ = store.find_many_by_subject(&subject).await?;
    let after_first_query = store.cache_stats();
    assert!(after_first_query.misses >= initial_stats.misses);
    assert!(store.cache.get(&subject).is_some());
    assert_eq!(store.cache.get(&subject).unwrap(), record);

    // Second query should hit the cache
    let _ = store.find_many_by_subject(&subject).await?;
    let after_second_query = store.cache_stats();
    assert!(after_second_query.hits >= after_first_query.hits);
    assert_eq!(after_second_query.misses, after_first_query.misses);
    assert_eq!(store.cache.get(&subject).unwrap(), record);

    Ok(())
}

#[tokio::test]
async fn test_cache_update_operations() -> StoreResult<()> {
    let config = CacheConfig {
        capacity: 100,
        ttl: Duration::from_secs(10),
        enabled: true,
    };
    let opts = DbConnectionOpts::default();
    let store = Store::<TestRecord>::with_cache_config(opts, config).await?;

    // Add initial record
    let subject = format!("{}.test.subject", create_random_db_name());
    let packet = TestRecord::new("initial payload").to_packet(&subject);
    store.add_record(&packet).await?;

    // Cache the record
    let _ = store.find_many_by_subject(&subject).await?;
    assert_eq!(store.cache.get(&subject).unwrap(), packet.record);

    // Update the record
    let updated_record = TestRecord::new("updated payload");
    let packet = updated_record.to_packet(&subject);
    store.update_record(&packet).await?;

    // Cache should be updated
    assert_eq!(store.cache.get(&subject).unwrap(), updated_record);

    // Verify cache hit when querying
    let result = store.find_many_by_subject(&subject).await?;
    assert_eq!(result[0], updated_record);

    Ok(())
}

#[tokio::test]
async fn test_cache_delete_operations() -> StoreResult<()> {
    let config = CacheConfig {
        capacity: 100,
        ttl: Duration::from_secs(10),
        enabled: true,
    };
    let opts = DbConnectionOpts::default();
    let store = Store::<TestRecord>::with_cache_config(opts, config).await?;

    // Add initial record
    let subject = format!("{}.test.subject", create_random_db_name());
    let packet = TestRecord::new("test payload").to_packet(&subject);
    store.add_record(&packet).await?;

    // Cache the record
    let _ = store.find_many_by_subject(&subject).await?;
    assert!(store.cache.get(&subject).is_some());

    // Delete the record
    store.delete_record(&packet).await?;

    // Cache should no longer have the record
    assert!(store.cache.get(&subject).is_none());

    // Verify record is gone from both cache and store
    let result = store.find_many_by_subject(&subject).await?;
    assert!(result.is_empty());

    Ok(())
}
