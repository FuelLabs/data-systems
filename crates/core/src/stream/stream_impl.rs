use std::{sync::Arc, time::Duration};

pub use async_nats::Subscriber as StreamLiveSubscriber;
use fuel_data_parser::DataEncoder;
use fuel_message_broker::NatsMessageBroker;
use fuel_streams_domains::{
    blocks::Block,
    infra::{
        db::{Db, DbItem},
        repository::Repository,
        QueryParamsBuilder,
    },
};
use fuel_streams_subject::subject::IntoSubject;
use fuel_streams_types::BlockHeight;
use fuel_web_utils::api_key::{ApiKeyRole, ApiKeyRoleScope};
use futures::{
    stream::{BoxStream, Stream as FStream},
    StreamExt,
};
use tokio::{sync::OnceCell, task::spawn_blocking, time::sleep};

use super::{config, StreamError};
use crate::{server::DeliverPolicy, types::StreamResponse};

pub type BoxedStoreItem = Result<StreamResponse, StreamError>;
pub type BoxedStream = Box<dyn FStream<Item = BoxedStoreItem> + Send + Unpin>;

#[derive(Debug, Clone)]
pub struct Stream<S: Repository> {
    db: Arc<Db>,
    broker: Arc<NatsMessageBroker>,
    namespace: Option<String>,
    _marker: std::marker::PhantomData<S>,
}

impl<R: Repository> Stream<R> {
    #[allow(clippy::declare_interior_mutable_const)]
    const INSTANCE: OnceCell<Self> = OnceCell::const_new();

    pub async fn get_or_init(
        broker: &Arc<NatsMessageBroker>,
        db: &Arc<Db>,
    ) -> Self {
        let cell = Self::INSTANCE;
        cell.get_or_init(|| async { Self::new(broker, db).await.to_owned() })
            .await
            .to_owned()
    }

    pub async fn new(broker: &Arc<NatsMessageBroker>, db: &Arc<Db>) -> Self {
        let broker = Arc::clone(broker);
        Self {
            db: db.clone(),
            broker,
            namespace: None,
            _marker: std::marker::PhantomData,
        }
    }

    #[cfg(any(test, feature = "test-helpers"))]
    pub fn with_namespace(
        broker: &Arc<NatsMessageBroker>,
        db: &Arc<Db>,
        namespace: String,
    ) -> Self {
        let broker = Arc::clone(broker);
        Self {
            db: db.clone(),
            broker,
            namespace: Some(namespace),
            _marker: std::marker::PhantomData,
        }
    }

    pub fn arc(&self) -> Arc<Self> {
        Arc::new(self.to_owned())
    }

    pub async fn publish(
        &self,
        subject: &str,
        response: &StreamResponse,
    ) -> Result<(), StreamError> {
        let broker = self.broker.clone();
        let response = response.clone();
        let payload = response.encode_json()?;
        broker.publish(subject, payload.into()).await?;
        Ok(())
    }

    pub async fn subscribe<S: IntoSubject + Into<R::QueryParams> + Clone>(
        &self,
        subject: S,
        deliver_policy: DeliverPolicy,
        api_key_role: &ApiKeyRole,
    ) -> BoxStream<'static, Result<StreamResponse, StreamError>> {
        let subject = Arc::new(subject);
        self.subscribe_dynamic(subject, deliver_policy, api_key_role)
            .await
    }

    pub async fn subscribe_dynamic<
        S: IntoSubject + Into<R::QueryParams> + Clone,
    >(
        &self,
        subject: Arc<S>,
        deliver_policy: DeliverPolicy,
        api_key_role: &ApiKeyRole,
    ) -> BoxStream<'static, Result<StreamResponse, StreamError>> {
        let broker = self.broker.clone();
        let subject = subject.clone();
        let stream = self.clone();
        let role = api_key_role.clone();
        let stream = async_stream::try_stream! {
            match role.has_scopes(&[ApiKeyRoleScope::HistoricalData]) {
                Ok(_) => {
                    if let DeliverPolicy::FromBlock { block_height } = deliver_policy {
                        let mut historical = stream.historical_streaming(subject.to_owned(), Some(block_height), &role);
                        while let Some(result) = historical.next().await {
                            yield result?;
                            let throttle_time = *config::STREAM_THROTTLE_HISTORICAL;
                            sleep(Duration::from_millis(throttle_time as u64)).await;
                        }
                    }
                }
                Err(e) => {
                    tracing::error!("Error subscribing to stream: {}", e);
                    Err(StreamError::from(e))?;
                }
            }

            match role.has_scopes(&[ApiKeyRoleScope::LiveData]) {
                Ok(_) => {
                    let mut live = broker.subscribe(&subject.parse()).await?;
                    while let Some(msg) = live.next().await {
                        let msg = msg?;
                        let stream_response = spawn_blocking(move || StreamResponse::decode_json(&msg)).await??;
                        yield stream_response;
                        let throttle_time = *config::STREAM_THROTTLE_LIVE;
                        sleep(Duration::from_millis(throttle_time as u64)).await;
                    }
                }
                Err(e) => {
                    tracing::error!("Error subscribing to stream: {}", e);
                    Err(StreamError::from(e))?;
                }
            }
        };
        Box::pin(stream)
    }

    pub fn historical_streaming<
        S: IntoSubject + Into<R::QueryParams> + Clone,
    >(
        &self,
        subject: Arc<S>,
        from_block: Option<BlockHeight>,
        role: &ApiKeyRole,
    ) -> BoxStream<'static, Result<StreamResponse, StreamError>> {
        let db = self.db.clone();
        let role = role.clone();
        let mut params: R::QueryParams = (*subject).clone().into();
        if cfg!(any(test, feature = "test-helpers")) {
            params.with_namespace(self.namespace.clone());
        }

        let stream = async_stream::try_stream! {
            let mut current_height = from_block.unwrap_or_default();
            params.with_from_block(Some(current_height));

            let mut last_height = Block::find_last_block_height(&db, params.options()).await?;
                while current_height <= last_height {
                    let items = R::find_many(db.pool_ref(), &params).await?;
                for item in items {
                    let subject = item.subject_str();
                    let subject_id = item.subject_id();
                    let block_height = item.block_height();
                    role.validate_historical_limit(last_height, block_height)?;
                    let value = item.encoded_value()?;
                    let pointer = item.into();
                    let response = StreamResponse::new(subject, subject_id, &value, pointer.to_owned(), None)?;
                    yield response;
                    current_height = pointer.block_height;
                }
                params.increment_offset();
                // When we reach the last known height, we need to check if any new blocks
                // were produced while we were processing the previous ones
                if current_height == last_height {
                    let new_last_height = Block::find_last_block_height(&db, params.options()).await?;
                    if new_last_height > last_height {
                        // Reset current_height back to process the blocks we haven't seen yet
                        current_height = last_height;
                        last_height = new_last_height;
                    } else {
                        tracing::debug!("No new blocks found, stopping historical streaming on block {}", current_height);
                        break
                    }
                }
            }
        };
        Box::pin(stream)
    }
}
