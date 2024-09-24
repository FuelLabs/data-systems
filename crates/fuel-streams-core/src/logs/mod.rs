pub mod subjects;
pub mod types;

pub use subjects::*;
use types::*;

use crate::prelude::*;

impl StreamEncoder for Log {}
impl Streamable for Log {
    const NAME: &'static str = "logs";
    const WILDCARD_LIST: &'static [&'static str] = &[LogsSubject::WILDCARD];
}
