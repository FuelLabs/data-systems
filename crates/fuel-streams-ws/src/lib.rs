pub mod cli;
pub mod client;
pub mod config;
pub mod server;
pub mod systems;
pub mod telemetry;

use std::{env, sync::LazyLock};

pub static MAX_THREADS: LazyLock<usize> = LazyLock::new(|| {
    let available_cpus = num_cpus::get();
    let default_threads = 2 * available_cpus;

    env::var("MAX_THREADS")
        .ok()
        .and_then(|val| val.parse().ok())
        .unwrap_or(default_threads)
});
