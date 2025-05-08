use fuel_streams_core::types::BlockHeight;

use crate::errors::ConsumerError;

#[derive(Copy, Clone, Debug)]
pub enum ActionType {
    Store,
    Stream,
}

impl std::fmt::Display for ActionType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ActionType::Store => write!(f, "store"),
            ActionType::Stream => write!(f, "stream"),
        }
    }
}

#[derive(Debug)]
pub struct BlockStats {
    pub start_time: std::time::Instant,
    pub end_time: std::time::Instant,
    pub packet_count: usize,
    pub block_height: BlockHeight,
    pub error: Option<ConsumerError>,
    pub action_type: ActionType,
}

impl BlockStats {
    pub fn new(block_height: BlockHeight, action_type: ActionType) -> Self {
        Self {
            start_time: std::time::Instant::now(),
            end_time: std::time::Instant::now(),
            packet_count: 0,
            block_height,
            error: None,
            action_type,
        }
    }

    pub fn calculate_block_propagation_ms(&self) -> u64 {
        let current = std::time::Instant::now();
        let diff = current.duration_since(self.start_time);
        diff.as_millis() as u64
    }

    pub fn finish(mut self, packet_count: usize) -> Self {
        self.end_time = std::time::Instant::now();
        self.packet_count = packet_count;
        self
    }

    pub fn finish_with_error(mut self, err: ConsumerError) -> Self {
        self.error = Some(err);
        self
    }

    pub fn duration_millis(&self) -> u128 {
        self.end_time.duration_since(self.start_time).as_millis()
    }

    pub fn log_error(&self, error: &ConsumerError) {
        let height = &self.block_height;
        let action = match self.action_type {
            ActionType::Store => "store",
            ActionType::Stream => "stream",
        };

        tracing::error!(
            "Failed to {} block {}: {:?} (took {:?} ms)",
            action,
            height,
            error,
            self.duration_millis()
        );
    }

    pub fn log_success(&self, prefix: &str) {
        let height = &self.block_height;
        let action = match self.action_type {
            ActionType::Store => "Stored",
            ActionType::Stream => "Streamed",
        };

        tracing::info!(
            "{} {} block {} with {} packets (took {:?} ms)",
            prefix,
            action,
            height,
            self.packet_count,
            self.duration_millis()
        );
    }
}
