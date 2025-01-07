use crate::record::{Record, RecordOrder};

#[derive(Clone)]
pub struct StorePacket<R: Record> {
    pub record: R,
    pub subject: String,
    pub order: RecordOrder,
}
impl<R: Record> StorePacket<R> {
    pub fn new(record: &R, subject: String, order: RecordOrder) -> Self {
        Self {
            record: record.to_owned(),
            subject,
            order,
        }
    }
}
