use fuel_streams_core::prelude::*;

pub trait IdSubjectsMutator: Streamable {
    fn push_id_subjects(
        kind: IdentifierKind,
        subjects: &mut Vec<(Box<dyn IntoSubject>, &'static str)>,
        tag: Option<Bytes32>,
    );
}

#[macro_export]
macro_rules! impl_subjects_mutator {
    ($t:ty, $s: ty) => {
        impl IdSubjectsMutator for $t {
            fn push_id_subjects(
                kind: IdentifierKind,
                subjects: &mut Vec<(Box<dyn IntoSubject>, &'static str)>,
                tag: Option<Bytes32>,
            ) {
                if let Some(tag) = tag.clone() {
                    subjects.push((
                        <$s>::new()
                            .with_id_kind(Some(kind))
                            .with_id_value(Some(tag))
                            .boxed(),
                        <$s>::WILDCARD,
                    ));
                }
            }
        }
    };
}

impl_subjects_mutator!(Transaction, TransactionsByIdSubject);
impl_subjects_mutator!(Input, InputsByIdSubject);
impl_subjects_mutator!(Output, OutputsByIdSubject);
impl_subjects_mutator!(Receipt, ReceiptsByIdSubject);

pub fn add_predicate_subjects<T: Streamable + IdSubjectsMutator>(
    subjects: &mut Vec<(Box<dyn IntoSubject>, &'static str)>,
    predicate_tag: Option<Bytes32>,
) {
    T::push_id_subjects(IdentifierKind::PredicateID, subjects, predicate_tag);
}

pub fn add_script_subjects<T: Streamable + IdSubjectsMutator>(
    subjects: &mut Vec<(Box<dyn IntoSubject>, &'static str)>,
    script_tag: Option<Bytes32>,
) {
    T::push_id_subjects(IdentifierKind::ScriptID, subjects, script_tag);
}
