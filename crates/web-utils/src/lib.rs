pub mod server;
pub mod shutdown;
pub mod telemetry;
pub mod tracing;

use std::sync::LazyLock;

pub static MAX_WORKERS: LazyLock<usize> = LazyLock::new(|| {
    let available_cpus = num_cpus::get();
    let default_threads = 2 * available_cpus;

    dotenvy::var("MAX_WORKERS")
        .ok()
        .and_then(|val| val.parse().ok())
        .unwrap_or(default_threads)
});
