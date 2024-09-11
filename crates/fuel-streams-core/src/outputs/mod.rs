pub mod subjects;

pub use subjects::*;

use crate::prelude::*;

impl StreamEncoder for fuel_tx::output::Output {}
impl Streamable for fuel_tx::output::Output {
    const NAME: &'static str = "outputs";
    const WILDCARD_LIST: &'static [&'static str] = &[
        CoinSubject::WILDCARD,
        ContractSubject::WILDCARD,
        ChangeSubject::WILDCARD,
        VariableSubject::WILDCARD,
        ContractCreatedSubject::WILDCARD,
        OutputsSubject::WILDCARD,
        OutputsByIdSubject::WILDCARD,
    ];
}
