pub mod subjects;
pub mod types;

pub use subjects::*;

use super::types::*;
use crate::{DataEncoder, StreamError, Streamable};

impl DataEncoder for Output {
    type Err = StreamError;
}
impl Streamable for Output {
    const NAME: &'static str = "outputs";
    const WILDCARD_LIST: &'static [&'static str] = &[
        OutputsByIdSubject::WILDCARD,
        OutputsCoinSubject::WILDCARD,
        OutputsContractSubject::WILDCARD,
        OutputsChangeSubject::WILDCARD,
        OutputsVariableSubject::WILDCARD,
        OutputsContractCreatedSubject::WILDCARD,
    ];
}
