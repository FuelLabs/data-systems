pub mod subjects;
pub mod types;

pub use subjects::*;
use types::*;

use crate::prelude::*;

impl StreamEncoder for Block {}
impl Streamable for Block {
    const NAME: &'static str = "blocks";
    const WILDCARD_LIST: &'static [&'static str] = &[BlocksSubject::WILDCARD];
}
