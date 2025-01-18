#[derive(Debug, Clone)]
pub struct QueryOptions {
    pub offset: i64,
    pub limit: i64,
    pub from_block: Option<u64>,
    pub to_block: Option<u64>,
    pub namespace: Option<String>,
}
impl Default for QueryOptions {
    fn default() -> Self {
        Self {
            offset: 0,
            limit: 100,
            from_block: None,
            to_block: None,
            namespace: None,
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
    pub fn with_from_block(mut self, from_block: Option<u64>) -> Self {
        self.from_block = from_block;
        self
    }
    pub fn with_namespace(mut self, namespace: Option<String>) -> Self {
        self.namespace = namespace;
        self
    }
    pub fn increment_offset(&mut self) {
        self.offset += self.limit;
    }
    pub fn with_to_block(mut self, to_block: Option<u64>) -> Self {
        self.to_block = to_block;
        self
    }
}
