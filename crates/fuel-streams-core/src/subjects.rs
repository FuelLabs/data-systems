pub use fuel_streams_macros::subject::*;

use crate::primitive_types::*;
pub use crate::{
    blocks::subjects::*,
    inputs::subjects::*,
    logs::subjects::*,
    outputs::subjects::*,
    receipts::subjects::*,
    transactions::subjects::*,
    utxos::subjects::*,
};

// ------------------------------------------------------------------------
// Identifier
// ------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub enum IdentifierKind {
    Address,
    ContractID,
    AssetID,
    PredicateID,
    ScriptID,
}

impl std::fmt::Display for IdentifierKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let value: &'static str = match self {
            IdentifierKind::Address => "address",
            IdentifierKind::ContractID => "contract_id",
            IdentifierKind::AssetID => "asset_id",
            IdentifierKind::PredicateID => "predicate_id",
            IdentifierKind::ScriptID => "script_id",
        };
        write!(f, "{value}")
    }
}

use std::sync::Arc;

#[derive(Debug, Clone)]
pub enum Identifier {
    Address(Bytes32, u8, Bytes32),
    ContractID(Bytes32, u8, Bytes32),
    AssetID(Bytes32, u8, Bytes32),
    PredicateID(Bytes32, u8, Bytes32),
    ScriptID(Bytes32, u8, Bytes32),
}

/// Macro to implement `From<Identifier>` for a given subject.
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
macro_rules! impl_from_identifier_for {
    ($subject:ident) => {
        impl From<Identifier> for Arc<$subject> {
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
                .arc()
            }
        }
    };
}

#[allow(clippy::unconditional_recursion)]
impl From<Identifier> for Arc<dyn IntoSubject> {
    fn from(identifier: Identifier) -> Self {
        identifier.into()
    }
}

impl_from_identifier_for!(TransactionsByIdSubject);
impl_from_identifier_for!(InputsByIdSubject);
impl_from_identifier_for!(OutputsByIdSubject);
impl_from_identifier_for!(ReceiptsByIdSubject);
