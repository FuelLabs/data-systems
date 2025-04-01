use fuel_streams_types::BlockHeight;
use serde::{Deserialize, Serialize};
#[derive(
    Debug, Clone, Default, Serialize, Deserialize, PartialEq, utoipa::ToSchema,
)]
pub struct QueryOptions {
    pub from_block: Option<BlockHeight>,
    pub namespace: Option<String>,
}

impl QueryOptions {
    pub fn with_from_block(mut self, from_block: Option<BlockHeight>) -> Self {
        self.from_block = from_block;
        self
    }

    pub fn with_namespace(mut self, namespace: Option<String>) -> Self {
        self.namespace = namespace;
        self
    }
}
