use std::sync::Arc;

use fuel_streams_core::prelude::*;
use rayon::prelude::*;

use crate::packets::PublishPacket;

#[derive(Debug, Clone)]
pub enum Identifier {
    Address(Bytes32, u8, Bytes32),
    ContractID(Bytes32, u8, Bytes32),
    AssetID(Bytes32, u8, Bytes32),
    PredicateID(Bytes32, u8, Bytes32),
    ScriptID(Bytes32, u8, Bytes32),
}

pub trait IdsExtractable: Streamable {
    fn extract_ids(
        &self,
        chain_id: &ChainId,
        tx: &Transaction,
        index: u8,
    ) -> Vec<Identifier>;
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
                    Identifier::Address(tx_id, index, id) => $subject::build(
                        Some(tx_id),
                        Some(index),
                        Some(IdentifierKind::Address),
                        Some(id),
                    ),
                    Identifier::ContractID(tx_id, index, id) => {
                        $subject::build(
                            Some(tx_id),
                            Some(index),
                            Some(IdentifierKind::ContractID),
                            Some(id),
                        )
                    }
                    Identifier::AssetID(tx_id, index, id) => $subject::build(
                        Some(tx_id),
                        Some(index),
                        Some(IdentifierKind::AssetID),
                        Some(id),
                    ),
                    Identifier::PredicateID(tx_id, index, id) => {
                        $subject::build(
                            Some(tx_id),
                            Some(index),
                            Some(IdentifierKind::PredicateID),
                            Some(id),
                        )
                    }
                    Identifier::ScriptID(tx_id, index, id) => $subject::build(
                        Some(tx_id),
                        Some(index),
                        Some(IdentifierKind::ScriptID),
                        Some(id),
                    ),
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
