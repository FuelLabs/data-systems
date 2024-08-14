mod client_opts;
mod conn_id;
mod conn_streams;
mod error;
mod nats_client;
mod nats_namespace;

pub(crate) mod stream;

mod errors;
mod nats_client;
/// Houses shared APIs for interacting with NATS for fuel-streams-publisher and fuel-streams crates
/// As much as possible, the public interface/APIS should be agnostic of NATS. These can then be extended
/// in the fuel-streams-publisher and fuel-streams crates to provide a more opinionated API towards
/// their specific use-cases.
mod nats_client_opts;
mod nats_namespace;

pub use fuel_streams_macros::subject::*;
pub mod types;

pub use client_opts::*;
pub use conn_id::*;
pub use conn_streams::*;
pub use error::*;
pub use nats_client::*;
pub use nats_namespace::*;


pub use stream::*;
pub use errors::*;
pub use nats_client::*;
pub use nats_client_opts::*;
pub use nats_namespace::*;
