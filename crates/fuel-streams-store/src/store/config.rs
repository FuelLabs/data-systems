use std::sync::LazyLock;

pub static STORE_PAGINATION_LIMIT: LazyLock<i64> = LazyLock::new(|| {
    dotenvy::var("STORE_PAGINATION_LIMIT")
        .ok()
        .and_then(|val| val.parse().ok())
        .unwrap_or(100)
});
