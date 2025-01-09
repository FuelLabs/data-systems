use std::sync::Arc;

use fuel_streams_macros::subject::IntoSubject;

use crate::record::Record;

#[derive(Clone)]
pub struct RecordPacket<R: Record> {
    pub record: R,
    pub subject: Arc<dyn IntoSubject + Send + Sync + 'static>,
}

impl<R: Record> RecordPacket<R> {
    pub fn new<S>(record: &R, subject: S) -> Self
    where
        S: IntoSubject + Send + Sync + 'static,
    {
        Self {
            record: record.to_owned(),
            subject: Arc::new(subject),
        }
    }
}
