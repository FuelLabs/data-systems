//! Newtype wrappers that provide `Drop`-based allocation tracking
//! for types we don't own.

use std::{
    pin::Pin,
    task::{Context, Poll},
};

use fuel_core_services::stream::BoxStream;
use fuel_indexer_types::events::BlockEvent;
use fuel_receipts_manager::adapters::graphql_event_adapter::GraphqlFetcher;
use futures::Stream;

use crate::alloc_counter;

// ---------------------------------------------------------------------------
// TrackedStream
// ---------------------------------------------------------------------------

/// A `BoxStream` wrapper whose `Drop` proves the stream was actually
/// deallocated, not just replaced.
pub struct TrackedStream {
    inner: BoxStream<anyhow::Result<BlockEvent>>,
}

impl TrackedStream {
    pub fn new(inner: BoxStream<anyhow::Result<BlockEvent>>) -> Self {
        alloc_counter::inc(&alloc_counter::BLOCK_STREAM);
        Self { inner }
    }
}

impl Stream for TrackedStream {
    type Item = anyhow::Result<BlockEvent>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        Pin::new(&mut self.inner).poll_next(cx)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}

impl Drop for TrackedStream {
    fn drop(&mut self) {
        alloc_counter::dec(&alloc_counter::BLOCK_STREAM);
    }
}

// ---------------------------------------------------------------------------
// TrackedFetcher
// ---------------------------------------------------------------------------

/// A `GraphqlFetcher` wrapper whose `Drop` proves the fetcher (and any
/// resources it holds) was actually deallocated.
pub struct TrackedFetcher {
    inner: GraphqlFetcher,
}

impl TrackedFetcher {
    pub fn new(inner: GraphqlFetcher) -> Self {
        alloc_counter::inc(&alloc_counter::GRAPHQL_FETCHER);
        Self { inner }
    }

    /// Delegate to the inner fetcher.
    pub fn inner(&self) -> &GraphqlFetcher {
        &self.inner
    }
}

impl std::ops::Deref for TrackedFetcher {
    type Target = GraphqlFetcher;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl Drop for TrackedFetcher {
    fn drop(&mut self) {
        alloc_counter::dec(&alloc_counter::GRAPHQL_FETCHER);
    }
}
