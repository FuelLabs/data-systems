pub mod subjects;
pub mod types;

pub use subjects::*;

use super::types::*;
use crate::{DataEncoder, StreamError, Streamable};

impl DataEncoder for Input {
    type Err = StreamError;
}
impl Streamable for Input {
    const NAME: &'static str = "inputs";
    const WILDCARD_LIST: &'static [&'static str] = &[
        InputsCoinSubject::WILDCARD,
        InputsContractSubject::WILDCARD,
        InputsMessageSubject::WILDCARD,
        InputsByIdSubject::WILDCARD,
    ];
}
