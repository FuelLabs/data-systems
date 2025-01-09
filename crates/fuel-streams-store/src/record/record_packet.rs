use std::{fmt::Debug, sync::Arc};

use fuel_streams_macros::subject::IntoSubject;

use crate::record::Record;

#[derive(Debug, thiserror::Error)]
pub enum RecordPacketError {
    #[error("Failed to downcast subject")]
    DowncastError,
}

#[derive(Debug, Clone)]
pub struct RecordPacket<R: Record> {
    pub record: Arc<R>,
    pub subject: Arc<dyn IntoSubject>,
    namespace: Option<String>,
}

impl<R: Record> RecordPacket<R> {
    pub fn new(subject: Arc<dyn IntoSubject>, record: &R) -> Self {
        Self {
            subject: Arc::clone(&subject),
            record: Arc::new(record.clone()),
            namespace: None,
        }
    }

    pub fn with_namespace(mut self, namespace: &str) -> Self {
        self.namespace = Some(namespace.to_string());
        self
    }

    pub fn subject_matches<S: IntoSubject + Clone>(
        &self,
    ) -> Result<S, RecordPacketError> {
        if let Some(subject) = self.subject.downcast_ref::<S>() {
            Ok(subject.clone())
        } else {
            Err(RecordPacketError::DowncastError)
        }
    }

    pub fn subject_str(&self) -> String {
        if cfg!(any(test, feature = "test-helpers")) {
            let mut subject = self.subject.parse();
            if let Some(namespace) = &self.namespace {
                subject = format!("{}.{}", namespace, subject);
            }
            subject
        } else {
            self.subject.parse()
        }
    }

    pub fn namespace(&self) -> Option<&str> {
        self.namespace.as_deref()
    }

    pub fn arc(&self) -> Arc<Self> {
        Arc::new(self.clone())
    }
}

/// Example usage:
/// ```rust
/// try_packet_subject_match!(packet, {
///     BlocksSubject => subject => {
///         Ok(BlockDbItem {
///             subject: subject.parse(),
///             value: record.encode_json().expect("Failed to encode block"),
///             height: record.height,
///             producer_address: record.producer.to_string(),
///         })
///     }
/// })
/// ```
///
/// The macro attempts to match the packet's subject against each provided subject type.
/// For each match attempt, if successful, executes the associated code block with the matched subject.
/// If no matches are found, returns a DowncastError
#[macro_export]
macro_rules! try_packet_subject_match {
    ($packet:expr, {
        $($subject_type:ty => $param:ident => $block:expr),+ $(,)?
    }) => {
        {
            $(
                if let Ok($param) = $packet.subject_matches::<$subject_type>() {
                    let $param = $param.clone();
                    $block
                } else
            )*
            {
                Err(RecordPacketError::DowncastError)
            }
        }
    };
}
