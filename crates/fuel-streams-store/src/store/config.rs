use std::sync::LazyLock;

pub static STORE_PAGINATION_LIMIT: LazyLock<i64> = LazyLock::new(|| {
    dotenvy::var("STORE_PAGINATION_LIMIT")
        .ok()
        .and_then(|val| val.parse().ok())
        .unwrap_or(100)
});

pub static STORE_MAX_RETRIES: LazyLock<i64> = LazyLock::new(|| {
    dotenvy::var("STORE_MAX_RETRIES")
        .ok()
        .and_then(|val| val.parse().ok())
        .unwrap_or(3)
});

pub static STORE_INITIAL_BACKOFF_MS: LazyLock<u64> = LazyLock::new(|| {
    dotenvy::var("STORE_INITIAL_BACKOFF_MS")
        .ok()
        .and_then(|val| val.parse().ok())
        .unwrap_or(100)
});
