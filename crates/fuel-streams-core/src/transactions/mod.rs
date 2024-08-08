pub mod subjects;
pub mod types;

pub use subjects::*;
use types::*;

use crate::prelude::Storable;

impl Storable for Transaction {
    const STORE: &'static str = "transactions";
}
