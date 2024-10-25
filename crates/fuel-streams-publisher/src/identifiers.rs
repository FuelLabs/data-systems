use fuel_streams_core::prelude::*;
use rayon::prelude::*;

use crate::SubjectPayload;

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

/// Trait for extracting identifiers from a streamable entity.
///
/// This trait should be implemented by any entity that can have identifiers
/// extracted from it, facilitating the creation of subjects for publishing.
///
/// # Examples
///
/// ```rust
/// use fuel_streams_publisher::identifiers::{Identifier, IdsExtractable};
/// use fuel_streams_core::prelude::*;
///
/// struct ExampleEntity;
///
/// impl IdsExtractable for ExampleEntity {
///     fn extract_identifiers(&self, _tx: &Transaction) -> Vec<Identifier> {
///         vec![Identifier::Address(Bytes32::zeroed())]
///     }
/// }
/// ```
pub trait IdsExtractable: Streamable {
    fn extract_identifiers(&self, tx: &Transaction) -> Vec<Identifier>;
}

/// Trait for building subject payloads based on extracted identifiers.
///
/// This trait leverages the identifiers extracted from entities to build
/// subjects that are used for publishing to streams.
///
/// # Examples
///
/// ```rust
/// use fuel_streams_publisher::identifiers::*;
/// use fuel_streams_core::prelude::*;
/// use rayon::prelude::*;
///
/// impl SubjectPayloadBuilder for OutputsByIdSubject {
///     fn build_subjects_payload<T: IdsExtractable>(
///         tx: &Transaction,
///         items: &[&T],
///     ) -> Vec<SubjectPayload> {
///         items
///             .par_iter()
///             .flat_map(|item| item.extract_identifiers(tx))
///             .map(|identifier| {
///                 // Example conversion from identifier to subject
///                 (identifier.into(), OutputsByIdSubject::WILDCARD)
///             })
///             .collect()
///     }
/// }
/// ```
pub trait SubjectPayloadBuilder: IntoSubject {
    fn build_subjects_payload<T: IdsExtractable>(
        tx: &Transaction,
        items: &[&T],
    ) -> Vec<SubjectPayload>;
}

/// Macro to implement `From<Identifier>` and `SubjectPayloadBuilder` for a given subject.
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
/// From<Identifier> trait and the SubjectPayloadBuilder trait for various subjects,
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

        impl SubjectPayloadBuilder for $subject {
            fn build_subjects_payload<T: IdsExtractable>(
                tx: &Transaction,
                items: &[&T],
            ) -> Vec<SubjectPayload> {
                items
                    .into_par_iter()
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
