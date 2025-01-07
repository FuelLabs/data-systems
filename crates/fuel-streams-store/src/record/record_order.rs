use crate::db::OrderIntSize;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecordOrder {
    pub block: OrderIntSize,
    pub tx: Option<OrderIntSize>,
    pub record: Option<OrderIntSize>,
}

impl RecordOrder {
    pub fn new(block: u32, tx: Option<u32>, record: Option<u32>) -> Self {
        Self {
            block: block as i64,
            tx: tx.map(|tx| tx as i64),
            record: record.map(|record| record as i64),
        }
    }
    pub fn with_tx(self, tx: u32) -> Self {
        Self {
            tx: Some(tx as i64),
            ..self
        }
    }
    pub fn with_record(self, record: u32) -> Self {
        Self {
            record: Some(record as i64),
            ..self
        }
    }
}
