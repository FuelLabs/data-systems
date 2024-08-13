pub mod subjects;
pub mod types;

pub use subjects::*;
use types::*;

use crate::prelude::Storable;

impl Storable for Block {
    const STORE: &'static str = "blocks";
}
