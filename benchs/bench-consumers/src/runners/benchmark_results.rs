use std::time::{Duration, Instant};

static MSGS_COUNT: usize = 10000;

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct BenchmarkResult {
    pub name: String,
    pub message_count: usize,
    pub error_count: usize,
    start_time: Instant,
    pub elapsed_time: Option<Duration>,
    pub messages_per_second: Option<f64>,
}

impl BenchmarkResult {
    pub fn new(name: String) -> Self {
        Self {
            name,
            message_count: 0,
            error_count: 0,
            start_time: Instant::now(),
            elapsed_time: None,
            messages_per_second: None,
        }
    }

    pub fn increment_message_count(&mut self) {
        println!("incrementing");
        self.message_count += 1;
    }

    pub fn increment_error_count(&mut self) {
        self.error_count += 1;
    }

    pub fn finalize(&mut self) {
        let elapsed = self.start_time.elapsed();
        self.elapsed_time = Some(elapsed);
        self.messages_per_second =
            Some(self.message_count as f64 / elapsed.as_secs_f64());
    }

    pub fn is_complete(&self) -> bool {
        self.message_count + self.error_count >= MSGS_COUNT
    }

    pub fn print_result(&self) {
        println!("\n{}", "=".repeat(50));
        println!("Benchmark Results: {}", self.name);
        println!("{}", "=".repeat(50));
        println!("Total Messages: {}", self.message_count);
        println!("Total Errors: {}", self.error_count);
        println!("Elapsed Time: {:?}", self.elapsed_time.unwrap_or_default());
        println!(
            "Messages per Second: {:.2}",
            self.messages_per_second.unwrap_or_default()
        );
        println!("{}", "=".repeat(50));
    }
}
