pub mod cli;
pub mod client;
pub mod config;
pub mod server;
pub mod telemetry;

use std::sync::LazyLock;

pub static STREAMER_MAX_WORKERS: LazyLock<usize> = LazyLock::new(|| {
    let available_cpus = num_cpus::get();
    let default_threads = 2 * available_cpus;

    dotenvy::var("STREAMER_MAX_WORKERS")
        .ok()
        .and_then(|val| val.parse().ok())
        .unwrap_or(default_threads)
});
