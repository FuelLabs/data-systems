use fuel_streams_types::BlockHeight;
use rand::Rng;
use serde::{Deserialize, Serialize};

#[derive(
    Debug, Clone, Default, Serialize, Deserialize, PartialEq, utoipa::ToSchema,
)]
pub struct QueryOptions {
    pub from_block: Option<BlockHeight>,
    pub namespace: Option<String>,
}

impl QueryOptions {
    pub fn with_from_block(
        &mut self,
        from_block: Option<BlockHeight>,
    ) -> &mut Self {
        self.from_block = from_block;
        self
    }

    pub fn with_namespace(&mut self, namespace: Option<String>) -> &mut Self {
        self.namespace = namespace;
        self
    }

    #[cfg(any(test, feature = "test-helpers"))]
    pub fn with_random_namespace() -> Self {
        let namespace =
            format!("test_{}", rand::rng().random_range(0..1000000));
        let mut opts = Self::default();
        opts.with_namespace(Some(namespace));
        opts
    }

    #[cfg(any(test, feature = "test-helpers"))]
    pub fn random_namespace() -> String {
        format!("test_{}", rand::rng().random_range(0..1000000))
    }
}
