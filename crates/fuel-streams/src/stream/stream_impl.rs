use std::pin::Pin;

use fuel_streams_core::{
    prelude::{IntoSubject, SubjectBuildable},
    types::DeliverPolicy,
    Streamable,
    SubscriptionConfig,
};

use crate::{client::Client, stream::StreamError};

/// A filter for stream subjects.
///
/// This struct is used to build and represent filters for stream subjects.
#[derive(Debug, Clone)]
pub struct Filter<S: SubjectBuildable> {
    /// The subject to filter on.
    pub subject: S,
}

impl<S: SubjectBuildable> Filter<S> {
    /// Builds a new subject filter.
    ///
    /// # Returns
    ///
    /// Returns a new instance of the subject type `S`.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use fuel_streams::stream::Filter;
    /// use fuel_streams::blocks::BlocksSubject;
    ///
    /// let filter = Filter::<BlocksSubject>::build();
    /// ```
    pub fn build() -> S {
        S::new()
    }
}

/// Configuration options for a stream.
#[derive(Debug, Clone, Default)]
pub struct StreamConfig {
    /// The delivery policy for the stream.
    pub deliver_policy: DeliverPolicy,
}

/// Represents a stream of data.
///
/// This struct wraps a `fuel_streams_core::Stream` and provides methods for
/// subscribing to and filtering the stream.
#[derive(Debug, Clone)]
pub struct Stream<S: Streamable> {
    stream: fuel_streams_core::Stream<S>,
    filter_subjects: Vec<String>,
}

impl<S: Streamable> Stream<S> {
    /// Creates a new `Stream` instance.
    ///
    /// # Parameters
    ///
    /// * `client`: A reference to a `Client` instance used to establish the connection.
    ///
    /// # Returns
    ///
    /// Returns a new `Stream` instance.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use fuel_streams::types::FuelNetwork;
    /// use fuel_streams::client::Client;
    /// use fuel_streams::stream::Stream;
    /// use fuel_streams::blocks::Block;
    ///
    /// # async fn example() -> Result<(), fuel_streams::Error> {
    /// let client = Client::connect(FuelNetwork::Local).await?;
    /// let stream = Stream::<Block>::new(&client).await;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn new(client: &Client) -> Self {
        let stream = fuel_streams_core::Stream::<S>::get_or_init(
            &client.nats_conn,
            &client.s3_conn,
        )
        .await;
        Self {
            stream,
            filter_subjects: Vec::new(),
        }
    }

    /// Adds a filter to the stream.
    ///
    /// # Parameters
    ///
    /// * `filter`: An object that can be converted into a subject filter.
    ///
    /// # Returns
    ///
    /// Returns a reference to the `Stream` instance for method chaining.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use fuel_streams::types::FuelNetwork;
    /// use fuel_streams::client::Client;
    /// use fuel_streams::stream::{Stream, Filter};
    /// use fuel_streams::blocks::{Block, BlocksSubject};
    /// use fuel_streams::types::Address;
    ///
    /// # async fn example() -> Result<(), fuel_streams::Error> {
    /// # let client = Client::connect(FuelNetwork::Local).await?;
    /// # let mut stream = Stream::<Block>::new(&client).await;
    /// let filter = Filter::<BlocksSubject>::build()
    ///     .with_producer(Some(Address::zeroed()))
    ///     .with_height(Some(5.into()));
    /// stream.with_filter(filter);
    /// # Ok(())
    /// # }
    /// ```
    pub fn with_filter(&mut self, filter: impl IntoSubject) -> &Self {
        self.filter_subjects.push(filter.parse());
        self
    }

    /// Subscribes to the stream item.
    ///
    /// # Returns
    ///
    /// Returns a `Result` containing a `futures::Stream` of byte vectors on success,
    /// or a `StreamError` on failure.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use fuel_streams::types::FuelNetwork;
    /// use fuel_streams::client::Client;
    /// use fuel_streams::stream::Stream;
    /// use fuel_streams::blocks::Block;
    ///
    /// # async fn example() -> Result<(), fuel_streams::Error> {
    /// # let client = Client::connect(FuelNetwork::Local).await?;
    /// # let stream = Stream::<Block>::new(&client).await;
    /// let subscription = stream.subscribe().await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn subscribe<'a>(
        &'a self,
    ) -> Result<Pin<Box<dyn futures::Stream<Item = S> + Send + 'a>>, StreamError>
    {
        // TODO: Why implicitly select a stream for the user?
        // TODO: Should this be a combination of streams
        self.stream
        // TODO: Improve DX by ensuring the stream returns the streamable entity directly
            .subscribe(None)
            .await
            .map_err(|source| StreamError::Subscribe { source })
    }

    /// Subscribes to the stream bytes.
    ///
    /// # Returns
    ///
    /// Returns a `Result` containing a `futures::Stream` of byte vectors on success,
    /// or a `StreamError` on failure.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use fuel_streams::types::FuelNetwork;
    /// use fuel_streams::client::Client;
    /// use fuel_streams::stream::Stream;
    /// use fuel_streams::blocks::Block;
    ///
    /// # async fn example() -> Result<(), fuel_streams::Error> {
    /// # let client = Client::connect(FuelNetwork::Local).await?;
    /// # let stream = Stream::<Block>::new(&client).await;
    /// let subscription = stream.subscribe().await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn subscribe_raw<'a>(
        &'a self,
    ) -> Result<
        Pin<Box<dyn futures::Stream<Item = Vec<u8>> + Send + 'a>>,
        StreamError,
    > {
        // TODO: Why implicitly select a stream for the user?
        // TODO: Should this be a combination of streams
        self.stream
        // TODO: Improve DX by ensuring the stream returns the streamable entity directly
            .subscribe_raw(None)
            .await
            .map_err(|source| StreamError::Subscribe { source })
    }

    /// Subscribes to the stream item with custom configuration options.
    ///
    /// # Parameters
    ///
    /// * `opts`: A `StreamConfig` instance containing custom configuration options.
    ///
    /// # Returns
    ///
    /// Returns a `Result` containing a `PullConsumerStream` on success,
    /// or a `StreamError` on failure.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use fuel_streams::types::FuelNetwork;
    /// use fuel_streams::client::Client;
    /// use fuel_streams::stream::{Stream, StreamConfig};
    /// use fuel_streams::blocks::Block;
    /// use fuel_streams::types::DeliverPolicy;
    ///
    /// # async fn example() -> Result<(), fuel_streams::Error> {
    /// # let client = Client::connect(FuelNetwork::Local).await?;
    /// # let stream = Stream::<Block>::new(&client).await;
    /// let config = StreamConfig {
    ///     deliver_policy: DeliverPolicy::All,
    /// };
    /// let subscription = stream.subscribe_with_config(config).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn subscribe_with_config<'a>(
        &'a self,
        opts: StreamConfig,
    ) -> Result<Pin<Box<dyn futures::Stream<Item = S> + Send + 'a>>, StreamError>
    {
        self.stream
        // TODO: Improve DX by ensuring the stream returns the streamable entity directly
            .subscribe(Some(SubscriptionConfig {
                deliver_policy: opts.deliver_policy,
                filter_subjects: self.filter_subjects.to_owned(),
            }))
            .await
            .map_err(|source| StreamError::SubscribeWithOpts { source })
    }

    /// Subscribes to the stream bytes with custom configuration options.
    ///
    /// # Parameters
    ///
    /// * `opts`: A `StreamConfig` instance containing custom configuration options.
    ///
    /// # Returns
    ///
    /// Returns a `Result` containing a `PullConsumerStream` on success,
    /// or a `StreamError` on failure.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use fuel_streams::types::FuelNetwork;
    /// use fuel_streams::client::Client;
    /// use fuel_streams::stream::{Stream, StreamConfig};
    /// use fuel_streams::blocks::Block;
    /// use fuel_streams::types::DeliverPolicy;
    ///
    /// # async fn example() -> Result<(), fuel_streams::Error> {
    /// # let client = Client::connect(FuelNetwork::Local).await?;
    /// # let stream = Stream::<Block>::new(&client).await;
    /// let config = StreamConfig {
    ///     deliver_policy: DeliverPolicy::All,
    /// };
    /// let subscription = stream.subscribe_with_config(config).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn subscribe_raw_with_config<'a>(
        &'a self,
        opts: StreamConfig,
    ) -> Result<
        Pin<Box<dyn futures::Stream<Item = Vec<u8>> + Send + 'a>>,
        StreamError,
    > {
        self.stream
        // TODO: Improve DX by ensuring the stream returns the streamable entity directly
            .subscribe_raw(Some(SubscriptionConfig {
                deliver_policy: opts.deliver_policy,
                filter_subjects: self.filter_subjects.to_owned(),
            }))
            .await
            .map_err(|source| StreamError::SubscribeWithOpts { source })
    }

    /// Returns a reference to the underlying `fuel_streams_core::Stream`.
    ///
    /// This method is only available when compiled with the `test` or `test-helpers` feature.
    ///
    /// # Returns
    ///
    /// Returns a reference to the underlying `fuel_streams_core::Stream<S>`.
    #[cfg(any(test, feature = "test-helpers"))]
    pub fn stream(&self) -> &fuel_streams_core::Stream<S> {
        &self.stream
    }
}
