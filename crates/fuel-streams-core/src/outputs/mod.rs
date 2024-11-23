pub mod subjects;
pub mod types;

pub use subjects::*;

use super::types::*;
use crate::{StreamEncoder, Streamable};

impl StreamEncoder for Output {}
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
