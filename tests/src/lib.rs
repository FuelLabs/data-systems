use fuel_streams_core::{stream::*, subjects::*};
use fuel_streams_nats::{NatsClient, NatsClientOpts};
use fuel_streams_store::{
    db::{Db, DbConnectionOpts, DbResult},
    impl_record_for,
    record::{Record, RecordEntity, RecordOrder},
    store::{Store, StorePacket},
};
use rand::Rng;
use serde::{Deserialize, Serialize};

// -----------------------------------------------------------------------------
// Setup
// -----------------------------------------------------------------------------

pub async fn setup_db() -> DbResult<Db> {
    let opts = DbConnectionOpts {
        pool_size: Some(10),
        ..Default::default()
    };
    Db::new(opts).await
}

pub async fn setup_store<R: Record>() -> DbResult<Store<R>> {
    let db = setup_db().await?;
    Ok(Store::new(&db.arc()))
}

pub async fn setup_nats(nats_url: &str) -> anyhow::Result<NatsClient> {
    let opts = NatsClientOpts::admin_opts().with_url(nats_url.to_string());
    let nats_client = NatsClient::connect(&opts).await?;
    Ok(nats_client)
}

pub async fn setup_stream(
    nats_url: &str,
) -> anyhow::Result<Stream<TestRecord>> {
    let nats_client = setup_nats(nats_url).await?;
    let db = setup_db().await?;
    let stream =
        Stream::<TestRecord>::get_or_init(&nats_client, &db.arc()).await;
    Ok(stream)
}

// -----------------------------------------------------------------------------
// Test data
// -----------------------------------------------------------------------------

#[derive(Subject, Debug, Clone, Default)]
#[subject_wildcard = "tests.>"]
#[subject_format = "tests.{name}"]
pub struct TestSubject {
    pub name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TestRecord(pub String);
impl_record_for!(TestRecord, RecordEntity::Block);
impl TestRecord {
    pub fn new(payload: impl Into<String>) -> Self {
        Self(payload.into())
    }
    pub fn to_packet(&self, subject: impl Into<String>) -> StorePacket<Self> {
        let order = RecordOrder::new(0, None, None);
        StorePacket::new(self, subject.into(), order)
    }
}

pub fn create_test_subject(name: impl Into<String>) -> TestSubject {
    TestSubject::build(Some(name.into()))
}

pub fn create_test_data(
    name: impl Into<String> + Clone,
) -> (TestSubject, TestRecord) {
    let subject = create_test_subject(name.clone());
    let record = TestRecord::new(name.into());
    (subject, record)
}

pub fn prefix_fn<R: Into<String>>() -> (String, impl Fn(R) -> String) {
    let prefix = create_random_db_name();
    (prefix.clone(), move |value: R| {
        format!("{}.{}", prefix, value.into())
    })
}

pub fn create_multiple_test_data(
    count: usize,
    name: impl Into<String> + Clone,
) -> Vec<(TestSubject, TestRecord)> {
    (0..count)
        .map(|idx| create_test_data(format!("{}.{}", name.clone().into(), idx)))
        .collect()
}

pub async fn add_test_records(
    store: &Store<TestRecord>,
    prefix: &str,
    records: &[(impl AsRef<str>, TestRecord)],
) -> anyhow::Result<()> {
    for (suffix, payload) in records {
        let subject = format!("{}.{}", prefix, suffix.as_ref());
        store.add_record(&payload.to_packet(&subject)).await?;
    }
    Ok(())
}

pub fn create_random_db_name() -> String {
    format!("test_{}", rand::thread_rng().gen_range(0..1000000))
}
