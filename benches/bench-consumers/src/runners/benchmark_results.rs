use core::fmt;
use std::time::{Duration, Instant};

use chrono::{DateTime, Utc};
use statrs::statistics::{Data, Distribution};

#[derive(Debug, Clone)]
pub struct BenchmarkResult {
    pub name: String,
    pub message_count: usize,
    pub error_count: usize,
    start_time: Instant,
    pub elapsed_time: Option<Duration>,
    pub messages_per_second: Option<f64>,
    pub publish_times: Vec<Duration>,
    pub mean_publish_time: Option<Duration>,
    pub messages_limit: usize,
}

impl fmt::Display for BenchmarkResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "\n{}\nBenchmark Results: {}\n{}\nTotal Messages: {}\nTotal Errors: {}\nElapsed Time: {:?}\nMessages per Second: {:.2}\nMean Publish Time: {:?}\n{}",
            "=".repeat(50),
            self.name,
            "=".repeat(50),
            self.message_count,
            self.error_count,
            self.elapsed_time.unwrap_or_default(),
            self.messages_per_second.unwrap_or_default(),
            self.mean_publish_time.unwrap_or_default(),
            "=".repeat(50)
        )
    }
}

impl BenchmarkResult {
    pub fn new(name: String, messages_limit: usize) -> Self {
        Self {
            name,
            message_count: 0,
            error_count: 0,
            start_time: Instant::now(),
            elapsed_time: None,
            messages_per_second: None,
            publish_times: vec![],
            mean_publish_time: None,
            messages_limit,
        }
    }

    pub fn increment_message_count(&mut self) {
        self.message_count += 1;
    }

    pub fn increment_error_count(&mut self) {
        self.error_count += 1;
    }

    pub fn finalize(&mut self) -> &mut Self {
        self.calculate_mean_publish_time();
        let elapsed = self.start_time.elapsed();
        self.elapsed_time = Some(elapsed);
        self.messages_per_second =
            Some(self.message_count as f64 / elapsed.as_secs_f64());
        self
    }

    pub fn is_complete(&self) -> bool {
        self.message_count + self.error_count >= self.messages_limit
    }

    pub fn add_publish_time(&mut self, timestamp: u128) -> &mut Self {
        let current_time = Utc::now();
        let publish_time =
            DateTime::<Utc>::from_timestamp_millis(timestamp as i64)
                .expect("Invalid timestamp");
        let duration = current_time
            .signed_duration_since(publish_time)
            .to_std()
            .expect("Duration calculation failed");

        self.publish_times.push(duration);
        self
    }

    pub fn calculate_mean_publish_time(&mut self) {
        if self.publish_times.is_empty() {
            return;
        }

        let times_ns: Vec<f64> = self
            .publish_times
            .iter()
            .map(|d| d.as_nanos() as f64)
            .collect();

        let data = Data::new(times_ns);
        let mean_ns = data.mean().unwrap();
        self.mean_publish_time = Some(Duration::from_nanos(mean_ns as u64));
    }
}
