#![deny(unused_crate_dependencies)]
#![deny(warnings)]

// Used in the `main.rs`
use fuel_web_utils as _;

pub mod alloc_counter;
pub mod block_buffer;
mod cli;
mod error;
pub mod helpers;
pub mod processor;
pub mod s3;
pub mod schemas;
pub mod service;
pub mod tracked;

pub use block_buffer::*;
pub use cli::*;
pub use error::*;
