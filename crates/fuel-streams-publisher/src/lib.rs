mod blocks;
pub mod cli;
mod inputs;
mod logs;
mod outputs;
mod publisher;
mod receipts;
mod transactions;
mod utxos;

mod fuel_core;

pub mod metrics;
pub mod server;
pub mod shutdown;
pub mod state;
pub mod system;

pub use fuel_core::{FuelCore, FuelCoreLike};
use fuel_streams_core::prelude::*;
pub use publisher::{Publisher, Streams};

fn build_subject_name(
    predicate_tag: &Option<Bytes32>,
    subject: &dyn IntoSubject,
) -> String {
    let subject_name = subject.parse();
    match predicate_tag {
        Some(tag) => format!("predicates.{tag}.{subject_name}"),
        None => subject_name,
    }
}
