pub mod subjects;
pub mod types;

pub use subjects::*;
use types::*;

use crate::prelude::*;

impl StreamEncoder for Input {}
impl Streamable for Input {
    const NAME: &'static str = "inputs";
    const WILDCARD_LIST: &'static [&'static str] = &[
        InputsCoinSubject::WILDCARD,
        InputsContractSubject::WILDCARD,
        InputsMessageSubject::WILDCARD,
    ];
}
