pub mod cli;
pub mod config;
pub mod metrics;
pub mod server;

use std::sync::LazyLock;

pub static STREAMER_MAX_WORKERS: LazyLock<usize> = LazyLock::new(|| {
    let available_cpus = num_cpus::get();
    let default_threads = 2 * available_cpus;

    dotenvy::var("STREAMER_MAX_WORKERS")
        .ok()
        .and_then(|val| val.parse().ok())
        .unwrap_or(default_threads)
});

pub static API_KEY_MAX_CONN_LIMIT: LazyLock<Option<u64>> =
    LazyLock::new(|| {
        dotenvy::var("API_KEY_MAX_CONN_LIMIT")
            .ok()
            .and_then(|val| val.parse::<u64>().ok())
    });
