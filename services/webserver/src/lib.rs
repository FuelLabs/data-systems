pub mod cli;
pub mod config;
pub mod metrics;
pub mod server;

use std::sync::LazyLock;

pub static API_PASSWORD: LazyLock<String> =
    LazyLock::new(|| dotenvy::var("API_PASSWORD").ok().unwrap_or_default());
