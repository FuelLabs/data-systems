pub(crate) mod blocks;
pub(crate) mod inputs;
pub(crate) mod logs;
pub(crate) mod outputs;
pub(crate) mod receipts;
pub(crate) mod transactions;
pub(crate) mod utxos;

pub use blocks::*;
pub use fuel_streams_macros::subject::*;
use fuel_streams_types::impl_from_identifier_for;
pub use inputs::*;
pub use logs::*;
pub use outputs::*;
pub use receipts::*;
pub use transactions::*;
pub use utxos::*;

impl_from_identifier_for!(TransactionsByIdSubject);
impl_from_identifier_for!(InputsByIdSubject);
impl_from_identifier_for!(OutputsByIdSubject);
impl_from_identifier_for!(ReceiptsByIdSubject);
