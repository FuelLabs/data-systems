pub mod subjects;
pub mod types;

pub use subjects::*;

use super::types::*;
use crate::{DataEncoder, StreamError, Streamable};

impl DataEncoder for Receipt {
    type Err = StreamError;
}
impl Streamable for Receipt {
    const NAME: &'static str = "receipts";
    const WILDCARD_LIST: &'static [&'static str] = &[
        ReceiptsCallSubject::WILDCARD,
        ReceiptsByIdSubject::WILDCARD,
        ReceiptsBurnSubject::WILDCARD,
        ReceiptsLogSubject::WILDCARD,
        ReceiptsMintSubject::WILDCARD,
        ReceiptsPanicSubject::WILDCARD,
        ReceiptsReturnSubject::WILDCARD,
        ReceiptsRevertSubject::WILDCARD,
        ReceiptsLogDataSubject::WILDCARD,
        ReceiptsTransferSubject::WILDCARD,
        ReceiptsMessageOutSubject::WILDCARD,
        ReceiptsReturnDataSubject::WILDCARD,
        ReceiptsTransferOutSubject::WILDCARD,
        ReceiptsScriptResultSubject::WILDCARD,
    ];
}
