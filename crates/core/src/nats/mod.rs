mod client_opts;
mod conn_id;
mod conn_streams;
mod errors;
mod nats_client;
mod nats_conn;
mod nats_namespace;

pub mod streams;
pub mod types;

pub use client_opts::*;
pub use conn_id::*;
pub use conn_streams::*;
pub use errors::*;
pub use nats_client::*;
pub use nats_conn::*;
pub use nats_namespace::*;
pub use streams::{stream::*, subject::*};

pub mod subjects {
    pub use streams::{
        stream_blocks::{subjects as blocks, BlockSubjects},
        stream_transactions::{subjects as transactions, TransactionSubjects},
    };

    use super::*;
}
