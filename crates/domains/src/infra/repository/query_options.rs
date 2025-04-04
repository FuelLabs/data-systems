use fuel_streams_types::{BlockHeight, BlockTimestamp};
use serde::{Deserialize, Serialize};

use crate::infra::TimeRange;

#[derive(
    Debug, Clone, Default, Serialize, Deserialize, PartialEq, utoipa::ToSchema,
)]
pub struct QueryOptions {
    pub from_block: Option<BlockHeight>,
    pub namespace: Option<String>,
    pub timestamp: Option<BlockTimestamp>,
    pub time_range: Option<TimeRange>,
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

    pub fn with_timestamp(
        &mut self,
        timestamp: Option<BlockTimestamp>,
    ) -> &mut Self {
        self.timestamp = timestamp;
        self
    }

    pub fn with_time_range(
        &mut self,
        time_range: Option<TimeRange>,
    ) -> &mut Self {
        self.time_range = time_range;
        self
    }

    #[cfg(any(test, feature = "test-helpers"))]
    pub fn with_random_namespace() -> Self {
        use rand::Rng;
        let namespace =
            format!("test_{}", rand::rng().random_range(0..1000000));
        let mut opts = Self::default();
        opts.with_namespace(Some(namespace));
        opts
    }

    #[cfg(any(test, feature = "test-helpers"))]
    pub fn random_namespace() -> String {
        use rand::Rng;
        format!("test_{}", rand::rng().random_range(0..1000000))
    }
}

pub trait HasOptions {
    fn options(&self) -> &QueryOptions;
}
