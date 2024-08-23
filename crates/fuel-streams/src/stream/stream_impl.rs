use fuel_streams_core::{
    prelude::IntoSubject,
    types::{DeliverPolicy, PullConsumerStream},
    Streamable,
    SubscribeConsumerConfig,
};

use crate::{client::Client, StreamError};

#[derive(Debug, Clone)]
pub struct Filter<S: IntoSubject> {
    pub subject: S,
}

impl<S: IntoSubject> Filter<S> {
    pub fn build() -> S {
        S::new()
    }
}

#[derive(Debug, Clone, Default)]
pub struct StreamConfig {
    pub deliver_policy: DeliverPolicy,
}

#[derive(Debug, Clone)]
pub struct Stream<S: Streamable> {
    stream: fuel_streams_core::Stream<S>,
    filter_subjects: Vec<String>,
}

impl<S: Streamable> Stream<S> {
    pub async fn new(client: &Client) -> Self {
        let stream =
            fuel_streams_core::Stream::<S>::get_or_init(&client.conn).await;
        Self {
            stream,
            filter_subjects: Vec::new(),
        }
    }

    pub fn with_filter(&mut self, filter: impl IntoSubject) -> &Self {
        self.filter_subjects.push(filter.parse());
        self
    }

    pub async fn subscribe(
        &self,
    ) -> Result<impl futures::Stream<Item = Vec<u8>>, StreamError> {
        // TODO: Why implicitly select a stream for the user?
        // TODO: Should this be a combination of streams
        self.stream
            .subscribe(S::WILDCARD_LIST[0])
            .await
            .map_err(|s| StreamError::Subscribe { source: s })
    }

    pub async fn subscribe_with_config(
        &self,
        opts: StreamConfig,
    ) -> Result<PullConsumerStream, StreamError> {
        self.stream
            .subscribe_consumer(SubscribeConsumerConfig {
                deliver_policy: opts.deliver_policy,
                filter_subjects: self.filter_subjects.to_owned(),
            })
            .await
            .map_err(|s| StreamError::SubscribeWithOpts { source: s })
    }

    #[cfg(any(test, feature = "test-helpers"))]
    pub fn stream(&self) -> &fuel_streams_core::Stream<S> {
        &self.stream
    }
}
