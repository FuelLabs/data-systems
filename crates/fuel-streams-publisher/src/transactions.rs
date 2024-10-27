use std::{iter::once, sync::Arc};

use fuel_core_storage::transactional::AtomicView;
use fuel_core_types::fuel_tx::field::ScriptData;
use fuel_streams_core::{prelude::*, transactions::TransactionExt};
use rayon::prelude::*;
use tokio::task::JoinHandle;

use crate::{
    identifiers::*,
    inputs::publish_tasks as publish_inputs,
    logs::publish_tasks as publish_logs,
    outputs::publish_tasks as publish_outputs,
    packets::{PublishError, PublishOpts, PublishPacket},
    receipts::publish_tasks as publish_receipts,
    sha256,
    utxos::publish_tasks as publish_utxos,
    FuelCoreLike,
    Streams,
};

pub fn publish_all_tasks(
    transactions: &[Transaction],
    streams: Streams,
    opts: &Arc<PublishOpts>,
    fuel_core: &dyn FuelCoreLike,
) -> Vec<JoinHandle<Result<(), PublishError>>> {
    transactions
        .iter()
        .enumerate()
        .flat_map(|tx_item| {
            let (_, tx) = tx_item;
            once(publish_tasks(
                tx_item,
                &streams.transactions,
                opts,
                fuel_core,
            ))
            .chain(once(publish_inputs(tx, &streams.inputs, opts)))
            .chain(once(publish_outputs(tx, &streams.outputs, opts)))
            .chain(once(publish_receipts(
                tx,
                &streams.receipts,
                opts,
                fuel_core,
            )))
            .chain(once(publish_logs(tx, &streams.logs, opts, fuel_core)))
            .chain(once(publish_utxos(tx, &streams.utxos, opts, fuel_core)))
            .flatten()
        })
        .collect()
}

fn publish_tasks(
    tx_item: (usize, &Transaction),
    stream: &Stream<Transaction>,
    opts: &Arc<PublishOpts>,
    fuel_core: &dyn FuelCoreLike,
) -> Vec<JoinHandle<Result<(), PublishError>>> {
    let (_, tx) = tx_item;
    let tx_id = tx.id(&opts.chain_id);
    let block_height = &opts.block_height;
    packets_from_tx(tx_item, tx_id, &opts.chain_id, fuel_core, block_height)
        .iter()
        .map(|packet| {
            packet.publish(Arc::new(stream.to_owned()), Arc::clone(opts))
        })
        .collect()
}

fn packets_from_tx(
    (index, tx): (usize, &Transaction),
    tx_id: fuel_core_types::fuel_tx::Bytes32,
    chain_id: &ChainId,
    fuel_core: &dyn FuelCoreLike,
    block_height: &BlockHeight,
) -> Vec<PublishPacket<Transaction>> {
    let kind = TransactionKind::from(tx.to_owned());
    let status: TransactionStatus = fuel_core
        .database()
        .off_chain()
        .latest_view()
        .unwrap()
        .get_tx_status(&tx_id)
        .unwrap()
        .map(|status| status.into())
        .unwrap_or_default();

    let receipts = fuel_core
        .get_receipts(&tx_id)
        .unwrap_or_default()
        .unwrap_or_default();

    let packets_from_inputs: Vec<PublishPacket<Transaction>> = tx
        .inputs()
        .par_iter()
        .flat_map(|item| {
            let ids = item.extract_ids(chain_id, tx, index as u8);
            tx.packets_from_ids(ids)
        })
        .collect();

    let packets_from_outputs: Vec<PublishPacket<Transaction>> = tx
        .outputs()
        .par_iter()
        .flat_map(|item| {
            let ids = item.extract_ids(chain_id, tx, index as u8);
            tx.packets_from_ids(ids)
        })
        .collect();

    let packets_from_receipts: Vec<PublishPacket<Transaction>> = receipts
        .par_iter()
        .flat_map(|item| {
            let ids = item.extract_ids(chain_id, tx, index as u8);
            tx.packets_from_ids(ids)
        })
        .collect();

    vec![PublishPacket::new(
        tx,
        TransactionsSubject::new()
            .with_tx_id(Some(tx_id.into()))
            .with_kind(Some(kind))
            .with_status(Some(status))
            .with_block_height(Some(block_height.to_owned()))
            .with_index(Some(index))
            .arc(),
        TransactionsSubject::WILDCARD,
    )]
    .into_iter()
    .chain(packets_from_inputs)
    .chain(packets_from_outputs)
    .chain(packets_from_receipts)
    .collect()
}

impl IdsExtractable for Transaction {
    fn extract_ids(
        &self,
        chain_id: &ChainId,
        tx: &Transaction,
        index: u8,
    ) -> Vec<Identifier> {
        let tx_id = tx.id(chain_id);
        match self {
            Transaction::Script(tx) => {
                let script_tag = sha256(tx.script_data());
                vec![Identifier::ScriptID(tx_id.into(), index, script_tag)]
            }
            _ => Vec::new(),
        }
    }
}
