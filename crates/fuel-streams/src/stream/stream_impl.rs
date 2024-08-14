use fuel_streams_core::{
    nats::{
        IntoSubject,
        StreamItem,
        Streamable,
        Streamer,
        SubscribeConsumerConfig,
    },
    types::{DeliverPolicy, PullConsumerStream},
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
    stream: Streamer<S>,
    filter_subjects: Vec<String>,
}

impl<S: Streamable> Stream<S> {
    pub async fn new(client: &Client) -> Result<Self, StreamError> {
        let stream = Streamer::<S>::get_or_init(&client.conn, None)
            .await
            .map_err(|s| StreamError::GetOrInitStream { source: s })?;
        Ok(Self {
            stream,
            filter_subjects: Vec::new(),
        })
    }

    pub fn with_filter(&mut self, filter: impl IntoSubject) -> &Self {
        self.filter_subjects.push(filter.parse());
        self
    }

    pub async fn subscribe(
        &self,
    ) -> Result<<S::Builder as StreamItem<S>>::Subscriber, StreamError> {
        let subject = S::MainSubject::all();
        self.stream
            .subscribe(subject)
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
    pub fn streamer(&self) -> &Streamer<S> {
        &self.stream
    }
}
