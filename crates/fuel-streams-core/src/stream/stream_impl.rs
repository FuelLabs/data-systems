use std::sync::{Arc, LazyLock};

use bytes::Bytes;
use fuel_streams_store::{
    db::{Db, DbRecord, Record},
    store::{Store, StorePacket},
};
use futures::{stream::BoxStream, StreamExt};
use tokio::sync::OnceCell;

use crate::prelude::*;

pub static MAX_ACK_PENDING: LazyLock<usize> = LazyLock::new(|| {
    dotenvy::var("MAX_ACK_PENDING")
        .ok()
        .and_then(|val| val.parse().ok())
        .unwrap_or(5)
});

#[derive(Debug, Clone)]
pub struct Stream<S: Record> {
    store: Arc<Store<S>>,
    nats_client: Arc<NatsClient>,
    _marker: std::marker::PhantomData<S>,
}

impl<R: Record> Stream<R> {
    #[allow(clippy::declare_interior_mutable_const)]
    const INSTANCE: OnceCell<Self> = OnceCell::const_new();

    pub async fn get_or_init(nats_client: &NatsClient, db: &Arc<Db>) -> Self {
        let cell = Self::INSTANCE;
        cell.get_or_init(|| async {
            Self::new(nats_client, db).await.to_owned()
        })
        .await
        .to_owned()
    }

    pub async fn new(nats_client: &NatsClient, db: &Arc<Db>) -> Self {
        let store = Arc::new(Store::new(db));
        let nats_client = Arc::new(nats_client.clone());
        Self {
            store,
            nats_client,
            _marker: std::marker::PhantomData,
        }
    }

    pub async fn publish(
        &self,
        packet: &StorePacket<R>,
    ) -> Result<DbRecord, StreamError> {
        let db_record = self.store.add_record(packet).await?;
        self.publish_to_nats(packet).await?;
        Ok(db_record)
    }

    async fn publish_to_nats(
        &self,
        packet: &StorePacket<R>,
    ) -> Result<(), StreamError> {
        let client = self.nats_client.nats_client.clone();
        let subject = packet.subject.clone();
        let payload = packet.record.encode().into();
        client.publish(subject, payload).await?;
        Ok(())
    }

    pub async fn subscribe_live<'a>(
        client: &Arc<NatsClient>,
        subject: String,
    ) -> Result<BoxStream<'a, (String, Bytes)>, StreamError> {
        let client = client.nats_client.clone();
        let subscriber = client.subscribe(subject.to_owned()).await?;
        Ok(subscriber
            .then(move |message| async move {
                (message.subject.to_string(), message.payload)
            })
            .boxed())
    }

    #[cfg(any(test, feature = "test-helpers"))]
    pub fn store(&self) -> &Store<R> {
        &self.store
    }

    pub fn arc(&self) -> Arc<Self> {
        Arc::new(self.to_owned())
    }
}

// #[cfg(any(test, feature = "test-helpers"))]
// mod tests {
//     use serde::{Deserialize, Serialize};

//     use super::*;

//     #[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
//     struct TestStreamable {
//         data: String,
//     }

//     impl DataEncoder for TestStreamable {
//         type Err = StreamError;
//     }

//     #[async_trait]
//     impl Streamable for TestStreamable {
//         const NAME: &'static str = "test_streamable";
//         const WILDCARD_LIST: &'static [&'static str] = &["*"];
//     }

//     #[tokio::test]
//     async fn test_stream_item_s3_encoding_flow() {
//         let (stream, _, test_data, subject) = setup_test().await;
//         let packet = test_data.to_packet(subject);

//         // Publish (this will encode and store in S3)
//         stream.publish(&packet).await.unwrap();

//         // Get the S3 path that was used
//         let s3_path = packet.get_s3_path();

//         // Retrieve directly from S3 and verify encoding
//         let raw_s3_data = stream.storage.retrieve(&s3_path).await.unwrap();
//         let decoded = TestStreamable::decode(&raw_s3_data).await.unwrap();
//         assert_eq!(decoded, test_data, "Retrieved data should match original");
//     }

//     #[tokio::test]
//     async fn test_stream_item_json_encoding_flow() {
//         let (_, _, test_data, _) = setup_test().await;
//         let encoded = test_data.encode().await.unwrap();
//         let decoded = TestStreamable::decode(&encoded).await.unwrap();
//         assert_eq!(decoded, test_data, "Decoded data should match original");

//         let json = DataParser::default().encode_json(&test_data).unwrap();
//         let json_str = String::from_utf8(json).unwrap();
//         let expected_json = r#"{"data":"test content"}"#;
//         assert_eq!(
//             json_str, expected_json,
//             "JSON structure should exactly match expected format"
//         );
//     }

//     #[cfg(test)]
//     async fn setup_test() -> (
//         Stream<TestStreamable>,
//         Arc<S3Storage>,
//         TestStreamable,
//         Arc<dyn IntoSubject>,
//     ) {
//         let storage = S3Storage::new_for_testing().await.unwrap();
//         let nats_client_opts =
//             NatsClientOpts::admin_opts().with_rdn_namespace();
//         let nats_client = NatsClient::connect(&nats_client_opts).await.unwrap();
//         let stream = Stream::<TestStreamable>::new(
//             &nats_client,
//             &Arc::new(storage.clone()),
//         )
//         .await;
//         let test_data = TestStreamable {
//             data: "test content".to_string(),
//         };
//         let subject = Arc::new(
//             BlocksSubject::new()
//                 .with_producer(Some(Address::zeroed()))
//                 .with_height(Some(1.into())),
//         );
//         (stream, Arc::new(storage), test_data, subject)
//     }
// }
