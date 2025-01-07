use crate::db::Record;

#[derive(Clone)]
pub struct StorePacket<R: Record> {
    pub record: R,
    pub subject: String,
    order: Option<i32>,
}
impl<R: Record> StorePacket<R> {
    pub fn new(record: &R, subject: String) -> Self {
        Self {
            record: record.to_owned(),
            subject,
            order: None,
        }
    }

    pub fn with_order(self, order: i32) -> Self {
        Self {
            order: Some(order),
            ..self
        }
    }

    pub fn order(&self) -> i32 {
        self.order.unwrap_or(0)
    }
}
