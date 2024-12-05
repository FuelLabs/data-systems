pub mod cli;
pub mod publisher;
pub mod server;
pub mod telemetry;

use std::{env, sync::LazyLock};

pub use publisher::*;

pub static PUBLISHER_MAX_THREADS: LazyLock<usize> = LazyLock::new(|| {
    let available_cpus = num_cpus::get();
    let default_threads = (available_cpus / 3).max(1); // Use 1/3 of CPUs, minimum 1

    env::var("PUBLISHER_MAX_THREADS")
        .ok()
        .and_then(|val| val.parse().ok())
        .unwrap_or(default_threads)
});

#[cfg(test)]
#[macro_use]
extern crate assert_matches;
