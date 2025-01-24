pub mod cli;
pub mod config;
pub mod metrics;
pub mod server;

use std::{sync::LazyLock, time::Duration};

pub static STREAMER_MAX_WORKERS: LazyLock<usize> = LazyLock::new(|| {
    let available_cpus = num_cpus::get();
    let default_threads = 2 * available_cpus;

    dotenvy::var("STREAMER_MAX_WORKERS")
        .ok()
        .and_then(|val| val.parse().ok())
        .unwrap_or(default_threads)
});

pub static API_PASSWORD: LazyLock<String> =
    LazyLock::new(|| dotenvy::var("API_PASSWORD").ok().unwrap_or_default());

pub static API_RATE_LIMIT_DURATION_MILLIS: LazyLock<Option<Duration>> =
    LazyLock::new(|| {
        dotenvy::var("API_RATE_LIMIT_DURATION_MILLIS")
            .ok()
            .and_then(|val| val.parse::<u64>().ok().map(Duration::from_millis))
    });
