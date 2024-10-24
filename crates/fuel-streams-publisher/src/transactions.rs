use fuel_core_storage::transactional::AtomicView;
use fuel_core_types::fuel_tx::field::ScriptData;
use fuel_streams_core::{prelude::*, transactions::TransactionExt};

use crate::{
    identifiers::*,
    sha256,
    FuelCoreLike,
    PublishPayload,
    SubjectPayload,
};

pub fn create_publish_payloads(
    stream: &Stream<Transaction>,
    tx: &Transaction,
    tx_index: usize,
    fuel_core: &dyn FuelCoreLike,
    chain_id: &ChainId,
    block_height: BlockHeight,
    receipts: Option<Vec<Receipt>>,
) -> Vec<PublishPayload<Transaction>> {
    let off_chain_database =
        fuel_core.database().off_chain().latest_view().unwrap();
    let tx_id = tx.id(chain_id);
    let kind = TransactionKind::from(tx.to_owned());
    let inputs = tx.inputs().to_vec();
    let outputs = tx.outputs().to_vec();
    let receipts = receipts.unwrap_or_default();
    let status: TransactionStatus = off_chain_database
        .get_tx_status(&tx_id)
        .unwrap()
        .map(|status| status.into())
        .unwrap_or_default();

    let subjects: Vec<SubjectPayload> = vec![(
        TransactionsSubject::new()
            .with_tx_id(Some(tx_id.into()))
            .with_kind(Some(kind))
            .with_status(Some(status))
            .with_block_height(Some(block_height))
            .with_tx_index(Some(tx_index))
            .boxed(),
        TransactionsSubject::WILDCARD,
    )];

    let subjects = subjects
        .into_iter()
        .chain(TransactionsByIdSubject::build_subjects_payload(
            tx,
            &[tx.clone()],
        ))
        .chain(TransactionsByIdSubject::build_subjects_payload(tx, &inputs))
        .chain(TransactionsByIdSubject::build_subjects_payload(
            tx, &outputs,
        ))
        .chain(TransactionsByIdSubject::build_subjects_payload(
            tx, &receipts,
        ))
        .collect();

    vec![PublishPayload {
        subjects,
        stream: stream.to_owned(),
        payload: tx.to_owned(),
    }]
}

impl IdsExtractable for Transaction {
    fn extract_identifiers(&self, _tx: &Transaction) -> Vec<Identifier> {
        match self {
            Transaction::Script(tx) => {
                let script_tag = sha256(tx.script_data());
                vec![Identifier::ScriptId(script_tag)]
            }
            _ => Vec::new(),
        }
    }
}
