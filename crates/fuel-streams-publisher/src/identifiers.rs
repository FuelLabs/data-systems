use fuel_streams_core::prelude::*;

use crate::SubjectPayload;

#[derive(Debug, Clone)]
pub enum Identifier {
    Address(Bytes32),
    ContractId(Bytes32),
    AssetId(Bytes32),
    PredicateId(Bytes32),
    ScriptId(Bytes32),
}

pub trait IdsExtractable: Streamable {
    fn extract_identifiers(&self, tx: &Transaction) -> Vec<Identifier>;
}

pub trait SubjectPayloadBuilder: IntoSubject {
    fn build_subjects_payload<T: IdsExtractable>(
        tx: &Transaction,
        item: &[T],
    ) -> Vec<SubjectPayload>;
}

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

        impl SubjectPayloadBuilder for $subject {
            fn build_subjects_payload<T: IdsExtractable>(
                tx: &Transaction,
                items: &[T],
            ) -> Vec<SubjectPayload> {
                items
                    .into_iter()
                    .flat_map(|item| item.extract_identifiers(tx))
                    .map(|identifier| {
                        let subject: $subject = identifier.into();
                        (
                            subject.boxed() as Box<dyn IntoSubject>,
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
