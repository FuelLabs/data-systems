//! Diagnostic allocation counters for tracking object lifetimes.
//!
//! Each counter is incremented on object creation and decremented **only** via
//! `Drop` impls, so a counter that trends upward over time is proof of a leak —
//! the object was created but never deallocated.
//!
//! Call [`log_all`] periodically to emit all live counts.

use std::sync::atomic::{AtomicI64, Ordering};

macro_rules! define_counters {
    ($($name:ident),+ $(,)?) => {
        $(pub static $name: AtomicI64 = AtomicI64::new(0);)+

        /// Log all counters at INFO level, including tokio task count.
        pub fn log_all() {
            let tokio_tasks = tokio::runtime::Handle::current()
                .metrics()
                .num_alive_tasks();
            tracing::info!(
                concat!("alloc_counters: ", $(stringify!($name), "={} ",)+ "TOKIO_TASKS={}"),
                $($name.load(Ordering::Relaxed),)+
                tokio_tasks,
            );
        }
    };
}

define_counters! {
    GRAPHQL_FETCHER,
    BLOCK_STREAM,
    AVRO_FILE_WRITERS,
    AVRO_FILE_WRITER,
    FINALIZED_BATCH_FILES,
}

/// Increment a counter (call from constructors).
#[inline]
pub fn inc(counter: &AtomicI64) {
    counter.fetch_add(1, Ordering::Relaxed);
}

/// Decrement a counter (call **only** from `Drop` impls).
#[inline]
pub fn dec(counter: &AtomicI64) {
    counter.fetch_sub(1, Ordering::Relaxed);
}
