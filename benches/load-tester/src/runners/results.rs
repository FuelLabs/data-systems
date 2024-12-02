use core::fmt;
use std::{
    sync::{
        atomic::{AtomicUsize, Ordering},
        Mutex,
    },
    time::{Duration, Instant},
};

use chrono::{DateTime, Utc};
use statrs::statistics::{Data, Distribution};

#[derive(Debug)]
pub struct LoadTestTracker {
    pub name: String,
    pub message_count: AtomicUsize,
    pub error_count: AtomicUsize,
    start_time: Instant,
    pub elapsed_time: Mutex<Option<Duration>>,
    pub messages_per_second: Mutex<Option<f64>>,
    pub publish_times: Mutex<Vec<Duration>>,
    pub mean_publish_time: Mutex<Option<Duration>>,
}

impl fmt::Display for LoadTestTracker {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "\n{}\nLoadTest Results: {}\n{}\nTotal Messages: {}\nTotal Errors: {}\nElapsed Time: {:?}\nMessages per Second: {:.2}\nMean Publish Time: {:?}\n{}",
            "=".repeat(50),
            self.name,
            "=".repeat(50),
            self.message_count.load(Ordering::Relaxed),
            self.error_count.load(Ordering::Relaxed),
            self.elapsed_time.lock().unwrap().unwrap_or_default(),
            self.messages_per_second.lock().unwrap().unwrap_or_default(),
            self.mean_publish_time.lock().unwrap().unwrap_or_default(),
            "=".repeat(50)
        )
    }
}

impl LoadTestTracker {
    pub fn new(name: String) -> Self {
        Self {
            name,
            message_count: AtomicUsize::new(0),
            error_count: AtomicUsize::new(0),
            start_time: Instant::now(),
            elapsed_time: Mutex::new(None),
            messages_per_second: Mutex::new(None),
            publish_times: Mutex::new(vec![]),
            mean_publish_time: Mutex::new(None),
        }
    }

    pub fn increment_message_count(&self) {
        self.message_count.fetch_add(1, Ordering::Relaxed);
    }

    pub fn increment_error_count(&self) {
        self.error_count.fetch_add(1, Ordering::Relaxed);
    }

    pub fn refresh(&self) -> &Self {
        self.calculate_mean_publish_time();

        let elapsed = self.start_time.elapsed();
        let message_count = self.message_count.load(Ordering::Relaxed);

        if let Ok(mut elapsed_time) = self.elapsed_time.lock() {
            *elapsed_time = Some(elapsed);
        }

        if let Ok(mut messages_per_second) = self.messages_per_second.lock() {
            *messages_per_second =
                Some(message_count as f64 / elapsed.as_secs_f64());
        }

        self
    }

    pub fn add_publish_time(&self, timestamp: u128) -> &Self {
        let current_time = Utc::now();
        let publish_time =
            DateTime::<Utc>::from_timestamp_millis(timestamp as i64)
                .expect("Invalid timestamp");
        let duration = current_time
            .signed_duration_since(publish_time)
            .to_std()
            .expect("Duration calculation failed");

        if let Ok(mut times) = self.publish_times.lock() {
            times.push(duration);
        }
        self
    }

    pub fn calculate_mean_publish_time(&self) {
        // Lock the mutex to access publish_times
        let times = self.publish_times.lock().unwrap();

        if times.is_empty() {
            return;
        }

        let times_ns: Vec<f64> =
            times.iter().map(|d| d.as_nanos() as f64).collect();
        drop(times);

        let data = Data::new(times_ns);
        let mean_ns = data.mean().unwrap();

        if let Ok(mut mean_publish_time) = self.mean_publish_time.lock() {
            *mean_publish_time = Some(Duration::from_nanos(mean_ns as u64));
        }
    }
}
