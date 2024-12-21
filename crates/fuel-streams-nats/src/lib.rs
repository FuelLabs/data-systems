/// Houses shared APIs for interacting with NATS for sv-publisher and fuel-streams crates
/// As much as possible, the public interface/APIS should be agnostic of NATS. These can then be extended
/// in the sv-publisher and fuel-streams crates to provide a more opinionated API towards
/// their specific use-cases.
pub mod error;
pub mod nats_client;
pub mod nats_client_opts;
pub mod nats_namespace;
pub mod types;

pub use error::*;
pub use nats_client::*;
pub use nats_client_opts::*;
pub use nats_namespace::*;
pub use types::*;
