mod client;
mod conn_streams;
mod errors;
mod nats_conn;

pub mod streams;
pub mod types;

pub use client::*;
pub use conn_streams::*;
pub use errors::*;
pub use nats_conn::*;

pub mod subjects {
    pub use streams::{
        stream_blocks::{subjects as blocks, BlockSubjects},
        stream_transactions::{subjects as transactions, TransactionSubjects},
    };

    use super::*;
}
