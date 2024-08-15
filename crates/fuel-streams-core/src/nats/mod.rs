/// Houses shared APIs for interacting with NATS for fuel-streams-publisher and fuel-streams crates
/// As much as possible, the public interface/APIS should be agnostic of NATS. These can then be extended
/// in the fuel-streams-publisher and fuel-streams crates to provide a more opinionated API towards
/// their specific use-cases.
mod error;
mod nats_client;
mod nats_client_opts;
mod nats_namespace;

pub mod types;

pub use error::*;
pub use nats_client::*;
pub use nats_client_opts::*;
pub use nats_namespace::*;
