use std::{fmt::Debug, str::FromStr, sync::Arc};

use fuel_streams_macros::subject::IntoSubject;

use super::{Record, RecordEntity};

#[derive(Debug, thiserror::Error)]
pub enum RecordPacketError {
    #[error("Failed to downcast subject")]
    DowncastError,
    #[error("Subject mismatch")]
    SubjectMismatch,
    #[error("Entity not found: {0}")]
    EntityNotFound(String),
}

pub trait PacketBuilder: Send + Sync + 'static {
    type Opts;
    fn build_packets(opts: &Self::Opts) -> Vec<RecordPacket>;
}

#[derive(Debug, Clone)]
pub struct RecordPacket {
    pub value: Vec<u8>,
    pub subject: Arc<dyn IntoSubject>,
    namespace: Option<String>,
}

impl RecordPacket {
    pub fn new(subject: Arc<dyn IntoSubject>, value: Vec<u8>) -> Self {
        Self {
            value,
            subject,
            namespace: None,
        }
    }

    pub fn arc(&self) -> Arc<Self> {
        Arc::new(self.to_owned())
    }

    pub fn to_record<R: Record>(&self) -> R {
        R::decode_json(&self.value)
            .unwrap_or_else(|_| panic!("Decoded failed for {}", R::ENTITY))
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

    pub fn get_entity(&self) -> Result<RecordEntity, RecordPacketError> {
        let subject_str = self.subject_str();
        let first_part = match self.namespace {
            Some(_) => subject_str.split('.').nth(1),
            None => subject_str.split('.').next(),
        };
        match first_part {
            Some(value) => RecordEntity::from_str(value)
                .map_err(RecordPacketError::EntityNotFound),
            _ => Err(RecordPacketError::EntityNotFound(
                "not_defined".to_string(),
            )),
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
}
