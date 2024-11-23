pub mod subjects;
pub mod types;

pub use subjects::*;

use super::types::*;
use crate::{StreamEncoder, Streamable};

impl StreamEncoder for Input {}
impl Streamable for Input {
    const NAME: &'static str = "inputs";
    const WILDCARD_LIST: &'static [&'static str] = &[
        InputsCoinSubject::WILDCARD,
        InputsContractSubject::WILDCARD,
        InputsMessageSubject::WILDCARD,
    ];
}
