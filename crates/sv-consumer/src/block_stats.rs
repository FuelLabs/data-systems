use fuel_streams_core::types::BlockHeight;

use crate::errors::ConsumerError;

#[derive(Debug)]
pub struct BlockStats {
    pub start_time: std::time::Instant,
    pub end_time: std::time::Instant,
    pub packet_count: usize,
    pub block_height: BlockHeight,
    pub error: Option<ConsumerError>,
}

impl BlockStats {
    pub fn new(block_height: BlockHeight) -> Self {
        Self {
            start_time: std::time::Instant::now(),
            end_time: std::time::Instant::now(),
            packet_count: 0,
            block_height,
            error: None,
        }
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
        tracing::error!(
            "Failed to process block {}: {:?} (took {:?} ms)",
            height,
            error,
            self.duration_millis()
        );
    }

    pub fn log_success(&self) {
        let height = &self.block_height;
        tracing::info!(
            "Processed block {} with {} packets (took {:?} ms)",
            height,
            self.packet_count,
            self.duration_millis()
        );
    }
}
