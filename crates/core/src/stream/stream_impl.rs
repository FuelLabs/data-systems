use std::{sync::Arc, time::Duration};

pub use async_nats::Subscriber as StreamLiveSubscriber;
use fuel_message_broker::NatsMessageBroker;
use fuel_streams_store::{
    db::{Db, DbItem},
    record::{DataEncoder, QueryOptions, Record},
    store::{find_last_block_height, Store},
};
use fuel_streams_subject::subject::IntoSubject;
use fuel_streams_types::BlockHeight;
use fuel_web_utils::api_key::{ApiKeyRole, ApiKeyRoleScope};
use futures::{
    stream::{BoxStream, Stream as FStream},
    StreamExt,
};
use tokio::{sync::OnceCell, task::spawn_blocking, time::sleep};

use super::{
    config::{STREAM_THROTTLE_HISTORICAL, STREAM_THROTTLE_LIVE},
    StreamError,
};
use crate::{server::DeliverPolicy, types::StreamResponse};

pub type BoxedStoreItem = Result<StreamResponse, StreamError>;
pub type BoxedStream = Box<dyn FStream<Item = BoxedStoreItem> + Send + Unpin>;

#[derive(Debug, Clone)]
pub struct Stream<S: Record> {
    store: Arc<Store<S>>,
    broker: Arc<NatsMessageBroker>,
    namespace: Option<String>,
    _marker: std::marker::PhantomData<S>,
}

impl<R: Record> Stream<R> {
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
        let store = Arc::new(Store::new(db));
        let broker = Arc::clone(broker);
        Self {
            store,
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
        let store = Arc::new(Store::new(db));
        let broker = Arc::clone(broker);
        Self {
            store,
            broker,
            namespace: Some(namespace),
            _marker: std::marker::PhantomData,
        }
    }

    pub fn store(&self) -> Store<R> {
        if let Some(namespace) = &self.namespace {
            let mut store = (*self.store).clone();
            store.with_namespace(&namespace.clone()).to_owned()
        } else {
            (*self.store).to_owned()
        }
    }

    pub fn arc(&self) -> Arc<Self> {
        Arc::new(self.to_owned())
    }

    pub async fn publish(
        &self,
        subject: &str,
        response: &Arc<StreamResponse>,
    ) -> Result<(), StreamError> {
        let broker = self.broker.clone();
        let response = response.clone();
        let payload = spawn_blocking(move || response.encode_json()).await??;
        broker.publish(subject, payload.into()).await?;
        Ok(())
    }

    pub async fn subscribe<S: IntoSubject>(
        &self,
        subject: S,
        deliver_policy: DeliverPolicy,
        api_key_role: &ApiKeyRole,
    ) -> BoxStream<'static, Result<StreamResponse, StreamError>> {
        let subject = Arc::new(subject);
        self.subscribe_dynamic(subject, deliver_policy, api_key_role)
            .await
    }

    pub async fn subscribe_dynamic(
        &self,
        subject: Arc<dyn IntoSubject>,
        deliver_policy: DeliverPolicy,
        api_key_role: &ApiKeyRole,
    ) -> BoxStream<'static, Result<StreamResponse, StreamError>> {
        let broker = self.broker.clone();
        let subject_ref = subject.clone();
        let stream = self.clone();
        let role = api_key_role.clone();
        let has_historical =
            role.has_scopes(&[ApiKeyRoleScope::HistoricalData]).is_ok();
        let has_live = role.has_scopes(&[ApiKeyRoleScope::LiveData]).is_ok();
        let throttle_historical = *STREAM_THROTTLE_HISTORICAL as u64;
        let throttle_live = *STREAM_THROTTLE_LIVE as u64;
        let stream = async_stream::try_stream! {
            if has_historical {
                if let DeliverPolicy::FromBlock { block_height } = deliver_policy {
                    let mut historical = stream.historical_streaming(subject_ref.to_owned(), Some(block_height), &role);
                    while let Some(result) = historical.next().await {
                        yield result?;
                        sleep(Duration::from_millis(throttle_historical)).await;
                    }
                }
            }

            if has_live {
                let mut live = broker.subscribe(&subject_ref.parse()).await?;
                while let Some(msg) = live.next().await {
                    let msg = msg?;
                    let stream_response = spawn_blocking(move || StreamResponse::decode_json(&msg))
                        .await?;
                    yield stream_response?;
                    sleep(Duration::from_millis(throttle_live)).await;
                }
            }
        };
        Box::pin(stream)
    }

    pub fn historical_streaming(
        &self,
        subject: Arc<dyn IntoSubject>,
        from_block: Option<BlockHeight>,
        role: &ApiKeyRole,
    ) -> BoxStream<'static, Result<StreamResponse, StreamError>> {
        let store = self.store();
        let db = store.db.clone();
        let role = role.clone();
        let opts = if cfg!(any(test, feature = "test-helpers")) {
            QueryOptions::default()
        } else {
            QueryOptions::default().with_namespace(self.namespace.clone())
        };

        let stream = async_stream::try_stream! {
            let mut current_height = from_block.unwrap_or_default();
            let mut last_height = find_last_block_height(&db, opts.clone()).await?;
            if let Err(e) = role.validate_historical_limit(last_height, current_height) {
                tracing::error!("Historical limit validation failed: {}", e);
                Err(StreamError::from(e))?;
            }

            let mut opts = opts.with_from_block(Some(current_height));
            while current_height <= last_height {
                let items = store.find_many_by_subject(&subject, opts.clone()).await?;
                if items.is_empty() {
                    let new_last_height = find_last_block_height(&db, opts.clone()).await?;
                    if new_last_height > last_height {
                        last_height = new_last_height;
                        continue;
                    }
                    tracing::debug!("No new blocks found, stopping historical streaming on block {}", current_height);
                    break;
                }

                for item in items {
                    let block_height = item.block_height();
                    let record_pointer = item.to_owned().into();
                    let response = StreamResponse::new(
                        item.subject_str(),
                        item.subject_id(),
                        item.encoded_value(),
                        record_pointer,
                        None,
                    )?;
                    yield response;
                    current_height = block_height;
                }
                opts = opts.with_from_block(Some(current_height));
            }
        };
        Box::pin(stream)
    }
}
