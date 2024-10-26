use std::sync::Arc;

use fuel_streams_core::prelude::*;
use rayon::prelude::*;

use crate::packets::PublishPacket;

/// Represents various types of identifiers used across different entities in the system.
///
/// # Examples
///
/// ```rust
/// use fuel_streams_publisher::identifiers::Identifier;
/// use fuel_streams_core::prelude::*;
///
/// let address = Identifier::Address(Bytes32::zeroed());
/// let contract_id = Identifier::ContractId(Bytes32::zeroed());
/// ```
#[derive(Debug, Clone)]
pub enum Identifier {
    Address(Bytes32),
    ContractId(Bytes32),
    AssetId(Bytes32),
    PredicateId(Bytes32),
    ScriptId(Bytes32),
}

pub trait IdsExtractable: Streamable {
    fn extract_ids(&self, tx: Option<&Transaction>) -> Vec<Identifier>;
}

pub trait PacketIdBuilder: Streamable {
    fn packets_from_ids(
        &self,
        ids: Vec<Identifier>,
    ) -> Vec<PublishPacket<Self>>;
}

/// Macro to implement `From<Identifier>` and `PacketBuilder` for a given subject.
///
/// This macro reduces boilerplate by automatically implementing the necessary
/// conversions and payload builders based on the provided entity type and subject.
///
/// This implementation is particularly important for the *ByIdSubjects in the project,
/// which are built using the Subject macro and utilize the builder pattern. Since these
/// subjects don't have a direct interface to use them as parameters, we need to create
/// this integration for each *ByIdSubject:
///     - TransactionsByIdSubject
///     - InputsByIdSubject
///     - OutputsByIdSubject
///     - ReceiptsByIdSubject
///
/// By using this macro, we ensure consistent and efficient implementation of the
/// From<Identifier> trait and the PacketBuilder trait for various subjects,
/// centralizing the logic of identity with data has inside the entity.
#[macro_export]
macro_rules! impl_subject_payload {
    ($entity: ty, $subject:ident) => {
        impl From<Identifier> for $subject {
            fn from(identifier: Identifier) -> Self {
                match identifier {
                    Identifier::Address(value) => $subject::new()
                        .with_id_kind(Some(IdentifierKind::Address))
                        .with_id_value(Some(value)),
                    Identifier::ContractId(value) => $subject::new()
                        .with_id_kind(Some(IdentifierKind::ContractID))
                        .with_id_value(Some(value)),
                    Identifier::AssetId(value) => $subject::new()
                        .with_id_kind(Some(IdentifierKind::AssetID))
                        .with_id_value(Some(value)),
                    Identifier::PredicateId(predicate_tag) => $subject::new()
                        .with_id_kind(Some(IdentifierKind::PredicateID))
                        .with_id_value(Some(predicate_tag)),
                    Identifier::ScriptId(script_tag) => $subject::new()
                        .with_id_kind(Some(IdentifierKind::ScriptID))
                        .with_id_value(Some(script_tag)),
                }
            }
        }

        impl PacketIdBuilder for $entity {
            fn packets_from_ids(
                &self,
                ids: Vec<Identifier>,
            ) -> Vec<PublishPacket<Self>> {
                ids.into_par_iter()
                    .map(|identifier| {
                        let subject: $subject = identifier.into();
                        PublishPacket::new(
                            self,
                            subject.arc() as Arc<dyn IntoSubject>,
                            $subject::WILDCARD,
                        )
                    })
                    .collect()
            }
        }
    };
}

impl_subject_payload!(Transaction, TransactionsByIdSubject);
impl_subject_payload!(Input, InputsByIdSubject);
impl_subject_payload!(Output, OutputsByIdSubject);
impl_subject_payload!(Receipt, ReceiptsByIdSubject);
