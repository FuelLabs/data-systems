use std::sync::LazyLock;

use fuel_streams_types::BlockHeight;

pub static STORE_PAGINATION_LIMIT: LazyLock<usize> = LazyLock::new(|| {
    dotenvy::var("STORE_PAGINATION_LIMIT")
        .ok()
        .and_then(|val| val.parse().ok())
        .unwrap_or(100)
});

#[derive(Debug, Clone)]
pub struct QueryOptions {
    pub offset: i64,
    pub limit: i64,
    pub from_block: Option<BlockHeight>,
    pub namespace: Option<String>,
    pub distinct: bool,
}
impl Default for QueryOptions {
    fn default() -> Self {
        Self {
            offset: 0,
            limit: *STORE_PAGINATION_LIMIT as i64,
            from_block: None,
            namespace: None,
            distinct: false,
        }
    }
}

impl QueryOptions {
    pub fn with_offset(mut self, offset: i64) -> Self {
        self.offset = offset.max(0);
        self
    }
    pub fn with_limit(mut self, limit: i64) -> Self {
        self.limit = limit.max(1);
        self
    }
    pub fn with_from_block(mut self, from_block: Option<BlockHeight>) -> Self {
        self.from_block = from_block;
        self
    }
    pub fn with_namespace(mut self, namespace: Option<String>) -> Self {
        self.namespace = namespace;
        self
    }
    pub fn with_distinct(mut self, distinct: bool) -> Self {
        self.distinct = distinct;
        self
    }
    pub fn increment_offset(&mut self) {
        self.offset += self.limit;
    }
}
