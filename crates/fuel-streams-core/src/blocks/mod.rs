pub mod subjects;
pub mod types;

use fuel_streams_macros::subject::IntoSubject;
pub use subjects::*;
use types::*;

use crate::prelude::*;

impl StreamEncoder for Block {}
impl Streamable for Block {
    const NAME: &'static str = "blocks";
    const WILDCARD_LIST: &'static [&'static str] = &[BlocksSubject::WILDCARD];

    type Builder = NatsStore<Self>;
    type MainSubject = BlocksSubject;
}
    
use crate::Streamable;

impl Streamable for Block {
    const STORE: &'static str = "blocks";
}
