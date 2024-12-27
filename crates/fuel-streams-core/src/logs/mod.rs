pub mod subjects;
pub mod types;

pub use subjects::*;

use super::types::*;
use crate::{DataEncoder, StreamError, Streamable};

impl DataEncoder for Log {
    type Err = StreamError;
}
impl Streamable for Log {
    const NAME: &'static str = "logs";
    const WILDCARD_LIST: &'static [&'static str] = &[LogsSubject::WILDCARD];
}
