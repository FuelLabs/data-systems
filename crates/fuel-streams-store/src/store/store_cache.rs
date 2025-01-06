use std::{
    sync::atomic::{AtomicU64, Ordering},
    time::Duration,
};

use moka::sync::Cache;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheConfig {
    pub capacity: u64,
    pub ttl: Duration,
    pub enabled: bool,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            capacity: 1000,
            ttl: Duration::from_secs(300), // 5 minutes
            enabled: true,
        }
    }
}

#[derive(Debug, Clone)]
pub struct CacheStats {
    pub hits: u64,
    pub misses: u64,
    pub size: u64,
}

pub struct StoreCache<T: Clone + Send + Sync + 'static> {
    cache: Cache<String, T>,
    config: CacheConfig,
    hits: AtomicU64,
    misses: AtomicU64,
}

impl<T: Clone + Send + Sync + 'static> StoreCache<T> {
    pub fn new(config: CacheConfig) -> Self {
        let cache = Cache::builder()
            .max_capacity(config.capacity)
            .time_to_live(config.ttl)
            .build();

        Self {
            cache,
            config,
            hits: AtomicU64::new(0),
            misses: AtomicU64::new(0),
        }
    }

    /// Returns whether the cache is enabled.
    /// This can be used to conditionally bypass the cache for testing or debugging purposes.
    #[allow(dead_code)]
    pub fn is_enabled(&self) -> bool {
        self.config.enabled
    }

    pub fn get(&self, subject: &str) -> Option<T> {
        if !self.config.enabled {
            return None;
        }
        match self.cache.get(subject) {
            Some(value) => {
                self.hits.fetch_add(1, Ordering::Relaxed);
                Some(value)
            }
            None => {
                self.misses.fetch_add(1, Ordering::Relaxed);
                None
            }
        }
    }

    pub fn insert(&self, subject: &str, value: &T) {
        if !self.config.enabled {
            return;
        }
        self.cache.insert(subject.to_string(), value.clone());
        self.cache.run_pending_tasks();
    }

    pub fn delete(&self, subject: &str) {
        if !self.config.enabled {
            return;
        }
        self.cache.remove(subject);
        self.cache.run_pending_tasks();
    }

    /// Invalidates all entries in the cache.
    /// This can be useful when you need to force a refresh of all cached data,
    /// for example during testing or when the underlying data source has changed significantly.
    #[allow(dead_code)]
    pub fn invalidate_all(&self) {
        if !self.config.enabled {
            return;
        }
        self.cache.invalidate_all();
        self.cache.run_pending_tasks();
    }

    pub fn stats(&self) -> CacheStats {
        CacheStats {
            hits: self.hits.load(Ordering::Relaxed),
            misses: self.misses.load(Ordering::Relaxed),
            size: self.cache.entry_count(),
        }
    }
}

#[cfg(test)]
mod tests {
    use tokio::time::sleep;

    use super::*;

    #[tokio::test]
    async fn test_cache_basic_operations() {
        let config = CacheConfig {
            capacity: 100,
            ttl: Duration::from_secs(1),
            enabled: true,
        };
        let cache = StoreCache::new(config);

        let subject = "test.subject".to_string();
        let value = "test value".to_string();

        // Test insert and get
        cache.insert(&subject, &value);
        let cached = cache.get(&subject).unwrap();
        assert_eq!(cached, value);

        // Test delete
        cache.delete(&subject);
        assert!(cache.get(&subject).is_none());

        // Test TTL
        cache.insert(&subject, &value);
        sleep(Duration::from_secs(2)).await;
        assert!(cache.get(&subject).is_none(), "Entry should expire");
    }

    #[tokio::test]
    async fn test_cache_disabled() {
        let config = CacheConfig {
            capacity: 100,
            ttl: Duration::from_secs(1),
            enabled: false,
        };
        let cache = StoreCache::new(config);

        let subject = "test.subject".to_string();
        let value = "test value".to_string();

        cache.insert(&subject, &value);
        assert!(cache.get(&subject).is_none(), "Cache should be disabled");
    }

    #[tokio::test]
    async fn test_cache_stats() {
        let config = CacheConfig::default();
        let cache = StoreCache::new(config);

        let subject = "test.subject".to_string();
        let value = "test value".to_string();

        // Initial stats
        let stats = cache.stats();
        assert_eq!(stats.hits, 0);
        assert_eq!(stats.misses, 0);
        assert_eq!(stats.size, 0);

        // Add entry and check stats
        cache.insert(&subject, &value);
        let stats = cache.stats();
        assert_eq!(stats.size, 1);

        // Get entry and check hits
        cache.get(&subject).unwrap();
        let stats = cache.stats();
        assert_eq!(stats.hits, 1);

        // Try getting non-existent entry
        cache.get("nonexistent");
        let stats = cache.stats();
        assert_eq!(stats.misses, 1);
    }
}
