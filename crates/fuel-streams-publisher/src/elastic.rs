use std::{fs, io, path::PathBuf, time::Duration};

use anyhow::Context;
use displaydoc::Display;
pub use elasticsearch::params::Refresh;
use elasticsearch::{
    self,
    auth::{ClientCertificate, Credentials},
    cert::{Certificate, CertificateValidation},
    http::{
        headers::HeaderMap,
        request::{Body, JsonBody},
        response::Response,
        transport::{SingleNodeConnectionPool, Transport, TransportBuilder},
        Method,
    },
    params,
    BulkParts,
    DeleteByQueryParts,
    Elasticsearch,
    IndexParts,
    SearchParts,
};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use thiserror::Error;
use url::{self, Url};

pub async fn create_elasticsearch_instance() -> anyhow::Result<ElasticSearch> {
    let elasticsearch_url = dotenvy::var("ELASTICSEARCH_URL")
        .expect("`ELASTICSEARCH_URL` env must be set");
    let elsaticsearch_username = dotenvy::var("ELASTICSEARCH_USERNAME")
        .expect("`ELASTICSEARCH_USERNAME` env must be set");
    let elsaticsearch_password = dotenvy::var("ELASTICSEARCH_PASSWORD")
        .expect("`ELASTICSEARCH_PASSWORD` env must be set");

    let config = Config {
        url: elasticsearch_url,
        enabled: true,
        pool_max_size: Some(2),
        username: Some(elsaticsearch_username),
        password: Some(elsaticsearch_password),
        ..Default::default()
    };
    let client = ElasticSearch::new(&config)
        .await
        .context("Failed to configure Elasticsearch connection")?;
    Ok(client)
}

/// Elasticsearch errors
#[derive(Debug, Display, Error)]
pub enum Error {
    /// ElasticSearchConfigError: `{0}`
    ElasticSearchConfigError(
        #[from] elasticsearch::http::transport::BuildError,
    ),
    /// ElasticSearchDisabled
    ElasticSearchDisabled,
    /// ElasticSearchError: `{0}`
    ElasticSearchError(#[from] elasticsearch::Error),
    /// IoError: `{0}`
    IoError(#[from] io::Error),
    /// UrlParseError: `{0}`
    UrlParseError(#[from] url::ParseError),
    /// CertificateError: `{0}`: `{0}`
    CertificateError(PathBuf, io::Error),
    /// SerdeJsonError: `{0}`
    SerdeJsonError(#[from] serde_json::Error),
}

#[derive(Debug, Deserialize, Clone, Default)]
#[serde(rename_all = "kebab-case")]
#[serde(default)]
pub struct Config {
    pub url: String,
    pub enabled: bool,
    pub username: Option<String>,
    pub password: Option<String>,
    pub api_key_id: Option<String>,
    pub api_key_value: Option<String>,
    pub pool_max_size: Option<usize>,
    pub pool_min_size: Option<usize>,
    pub tls: Option<TlsConfig>,
}

/// TLS acceptor configuration.
#[derive(Debug, Deserialize, Clone, Default)]
#[serde(rename_all = "kebab-case")]
#[serde(default)]
pub struct TlsConfig {
    /// Filename of CA certificates in PEM format.
    pub ca: Option<PathBuf>,
    /// Filename of combined TLS client certificate and key in PKCS#12 format.
    pub certificate: Option<PathBuf>,
    /// Optional passphrase to decode the TLS private key.
    pub key_passphrase: Option<String>,
}

// #[derive(Clone)]
pub struct ElasticSearch(ElasticConnection);

impl ElasticSearch {
    pub async fn new(config: &Config) -> Result<Self, Error> {
        if !config.enabled {
            return Err(Error::ElasticSearchDisabled);
        }
        let conn_info = ConnectionInfo::new(config)?;
        let conn = conn_info
            .get_connection()
            .expect("connection must be created");
        Ok(Self(conn))
    }

    pub fn get_conn(&self) -> &ElasticConnection {
        &self.0
    }

    pub async fn index<B>(
        &self,
        path: &str,
        id: Option<&str>,
        doc: B,
        refresh: Option<Refresh>,
    ) -> Result<(), Error>
    where
        B: Serialize,
    {
        self.get_conn().index(path, id, doc, refresh).await
    }

    pub async fn bulk_iter<B, I>(
        &self,
        path: Option<&str>,
        iter: I,
        refresh: Option<Refresh>,
    ) -> Result<BulkResults, Error>
    where
        B: Serialize,
        I: IntoIterator<Item = BulkItem<B>>,
    {
        self.get_conn().bulk_iter(path, iter, refresh).await
    }

    pub async fn query<B, Q>(
        &self,
        path: Option<&[&str]>,
        query_string: Option<&Q>,
        body: Option<B>,
    ) -> Result<SearchHits, Error>
    where
        B: Serialize,
        Q: Serialize + ?Sized,
    {
        self.get_conn().query(path, query_string, body).await
    }

    pub async fn delete_by_query<B>(
        &self,
        indices: &[&str],
        body: B,
    ) -> Result<u64, Error>
    where
        B: Serialize,
    {
        self.get_conn().delete_by_query(indices, body).await
    }
}

#[derive(Clone, Debug)]
pub struct SearchHits {
    pub total_hits: u64,
    pub hits: Option<Vec<JsonValue>>,
}

pub struct BulkItem<B: Serialize> {
    pub op: Operation<OperationParams>,
    pub doc: B,
}

impl<B> BulkItem<B>
where
    B: Serialize,
{
    pub fn new(op: Operation<OperationParams>, doc: B) -> Self {
        Self { op, doc }
    }
}

#[derive(Clone, Debug, PartialEq, Deserialize)]
pub struct BulkResults {
    pub errors: bool,
    #[serde(rename = "items")]
    pub results: Vec<Operation<OperationStatus>>,
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Operation<T> {
    Create(T),
    Delete(T),
    Index(T),
    Update(T),
}

#[derive(Clone, Debug, Default, PartialEq, Serialize)]
pub struct OperationParams {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    id: Option<String>,
    #[serde(rename = "_index", skip_serializing_if = "Option::is_none")]
    index: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    version: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    version_type: Option<params::VersionType>,
}

pub enum Version {
    External(u64),
    ExternalGte(u64),
}

impl OperationParams {
    pub fn builder() -> OperationsParamsBuilder {
        OperationsParamsBuilder::new()
    }
}

#[derive(Default)]
pub struct OperationsParamsBuilder(OperationParams);

impl OperationsParamsBuilder {
    pub fn new() -> Self {
        Self(OperationParams::default())
    }

    #[must_use]
    pub fn id(mut self, id: String) -> Self {
        self.0.id = Some(id);
        self
    }

    #[must_use]
    pub fn index(mut self, index: String) -> Self {
        self.0.index = Some(index);
        self
    }

    #[must_use]
    pub fn version(mut self, version: Version) -> Self {
        let (version_number, version_type) = match version {
            Version::External(num) => (num, params::VersionType::External),
            Version::ExternalGte(num) => {
                (num, params::VersionType::ExternalGte)
            }
        };
        self.0.version_type = Some(version_type);
        self.0.version = Some(version_number);
        self
    }

    pub fn build(self) -> OperationParams {
        self.0
    }
}

#[derive(Clone, Debug, PartialEq, Deserialize)]
pub struct OperationStatus {
    #[serde(rename = "_id")]
    pub id: Option<String>,
    #[serde(rename = "_index")]
    pub index: Option<String>,
    #[serde(rename = "status")]
    pub http_code: u32,
    #[serde(flatten)]
    pub result: OperationResult,
}

impl Operation<OperationStatus> {
    pub fn is_ok(&self) -> bool {
        matches!(self.status().result, OperationResult::Ok(_))
    }

    pub fn is_err(&self) -> bool {
        matches!(self.status().result, OperationResult::Error { .. })
    }

    pub fn status(&self) -> &OperationStatus {
        match self {
            Operation::Create(status)
            | Operation::Delete(status)
            | Operation::Index(status)
            | Operation::Update(status) => status,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Deserialize)]
pub enum OperationResult {
    #[serde(rename = "result")]
    Ok(String),
    #[serde(rename = "error")]
    Error {
        #[serde(rename = "type")]
        kind: String,
        reason: String,
    },
}

impl OperationResult {
    pub fn is_ok(&self) -> bool {
        matches!(self, OperationResult::Ok(_))
    }

    pub fn is_err(&self) -> bool {
        matches!(self, OperationResult::Error { .. })
    }
}

#[derive(Clone, Debug)]
pub struct ConnectionInfo(Transport);

impl ConnectionInfo {
    pub fn new(config: &Config) -> Result<Self, Error> {
        let url = Url::parse(&config.url)?;
        let pool = SingleNodeConnectionPool::new(url);
        let transport = TransportBuilder::new(pool);
        let tls = config.tls.clone().unwrap_or_default();
        let credentials = match (
            config.api_key_id.as_ref(),
            config.api_key_value.as_ref(),
            tls.certificate,
        ) {
            (Some(api_key_id), Some(api_key_value), _) => Some(
                Credentials::ApiKey(api_key_id.into(), api_key_value.into()),
            ),
            (_, _, Some(certificate)) => {
                Some(Credentials::Certificate(ClientCertificate::Pkcs12(
                    fs::read(&certificate).map_err(|err| {
                        Error::CertificateError(certificate, err)
                    })?,
                    tls.key_passphrase,
                )))
            }
            _ => config.username.as_ref().map(|username| {
                Credentials::Basic(
                    username.into(),
                    config.password.clone().unwrap_or_default(),
                )
            }),
        };
        let transport =
            if let Some(ca) = tls.ca {
                transport.cert_validation(CertificateValidation::Full(
                    Certificate::from_pem(&fs::read(&ca).map_err(|err| {
                        Error::CertificateError(ca.clone(), err)
                    })?)
                    .map_err(|err| {
                        Error::CertificateError(
                            ca,
                            io::Error::new(io::ErrorKind::Other, err),
                        )
                    })?,
                ))
            } else {
                transport
            };
        let transport = if let Some(credentials) = credentials {
            transport.auth(credentials)
        } else {
            transport
        };
        let inner = transport.build()?;
        Ok(Self(inner))
    }

    pub fn get_connection(&self) -> Result<ElasticConnection, Error> {
        let conn = Elasticsearch::new(self.0.clone());
        Ok(ElasticConnection(Some(conn)))
    }
}

pub struct ElasticConnection(Option<Elasticsearch>);

impl ElasticConnection {
    pub async fn connect(
        address: &ConnectionInfo,
    ) -> Result<ElasticConnection, Error> {
        address.get_connection()
    }

    pub fn check_alive(&self) -> Option<bool> {
        Some(self.0.is_some())
    }

    pub async fn ping(&self) -> Result<(), Error> {
        let conn = self.0.as_ref().ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::ConnectionAborted,
                "Connection to Elasticsearch is already closed",
            )
        })?;

        let response = conn.ping().send().await?;
        let _ = response.error_for_status_code()?;
        Ok(())
    }
}

impl ElasticConnection {
    pub async fn index<B>(
        &self,
        path: &str,
        id: Option<&str>,
        doc: B,
        refresh: Option<Refresh>,
    ) -> Result<(), Error>
    where
        B: Serialize,
    {
        let conn = self.0.as_ref().ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::ConnectionAborted,
                "Connection to Elasticsearch is already closed",
            )
        })?;
        let index_parts = id
            .map(|id| IndexParts::IndexId(path, id))
            .unwrap_or(IndexParts::Index(path));

        let response = conn
            .index(index_parts)
            .body(doc)
            .refresh(refresh.unwrap_or(Refresh::False))
            .send()
            .await?;
        response
            .error_for_status_code()
            .map(|_| ())
            .map_err(Into::into)
    }

    pub async fn bulk_iter<B, I>(
        &self,
        path: Option<&str>,
        iter: I,
        refresh: Option<Refresh>,
    ) -> Result<BulkResults, Error>
    where
        B: Serialize,
        I: IntoIterator<Item = BulkItem<B>>,
    {
        let conn = self.0.as_ref().ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::ConnectionAborted,
                "Connection to Elasticsearch is already closed",
            )
        })?;
        let body = build_bulk_request_body(iter)?;
        let bulk_parts = path.map(BulkParts::Index).unwrap_or(BulkParts::None);
        let response = conn
            .bulk(bulk_parts)
            .body(body)
            .refresh(refresh.unwrap_or(Refresh::False))
            .send()
            .await?;
        let response = response.error_for_status_code()?;
        response.json::<BulkResults>().await.map_err(Into::into)
    }

    pub async fn query<B, Q>(
        &self,
        path: Option<&[&str]>,
        query_string: Option<&Q>,
        body: Option<B>,
    ) -> Result<SearchHits, Error>
    where
        B: Serialize,
        Q: Serialize + ?Sized,
    {
        let response = self
            .send_query(path, query_string, body.map(JsonBody::from), None)
            .await?;
        let response = response.error_for_status_code()?;
        let mut body = response.json::<JsonValue>().await?;
        let hits: Option<Vec<_>> =
            body["hits"]["hits"].as_array_mut().map(|hits| {
                hits.iter_mut().map(|hit| hit["_source"].take()).collect()
            });
        let num_hits = hits.as_ref().map(Vec::len).unwrap_or(0);
        let total_hits = body["hits"]["total"]["value"]
            .as_u64()
            .unwrap_or(num_hits as u64);
        Ok(SearchHits { total_hits, hits })
    }

    async fn send_query<B, Q>(
        &self,
        path: Option<&[&str]>,
        query_string: Option<&Q>,
        body: Option<B>,
        timeout: Option<Duration>,
    ) -> Result<Response, Error>
    where
        B: Body,
        Q: Serialize + ?Sized,
    {
        let conn = self.0.as_ref().ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::ConnectionAborted,
                "Connection to Elasticsearch is already closed",
            )
        })?;
        let search_parts = match path {
            Some(path) => SearchParts::Index(path),
            None => SearchParts::None,
        };
        let response = conn
            .send(
                Method::Post,
                search_parts.url().as_ref(),
                HeaderMap::new(),
                query_string,
                body,
                timeout,
            )
            .await?;
        Ok(response)
    }

    pub async fn delete_by_query<B>(
        &self,
        indices: &[&str],
        body: B,
    ) -> Result<u64, Error>
    where
        B: Serialize,
    {
        let conn = self.0.as_ref().ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::ConnectionAborted,
                "Connection to Elasticsearch is already closed",
            )
        })?;

        let response = conn
            .delete_by_query(DeleteByQueryParts::Index(indices))
            .body(body)
            .wait_for_completion(true)
            .refresh(true)
            .send()
            .await?
            .error_for_status_code()?;
        let body = response.json::<JsonValue>().await?;
        let deleted = body["deleted"].as_u64().unwrap_or_default();
        Ok(deleted)
    }
}

fn build_bulk_request_body<B, I>(
    iter: I,
) -> Result<Vec<JsonBody<JsonValue>>, Error>
where
    B: Serialize,
    I: IntoIterator<Item = BulkItem<B>>,
{
    let mut body: Vec<JsonBody<JsonValue>> = Vec::new();
    for item in iter.into_iter() {
        body.push(serde_json::to_value(item.op)?.into());
        body.push(serde_json::to_value(item.doc)?.into());
    }
    Ok(body)
}

#[cfg(test)]
mod tests {
    use elasticsearch::http::request::{Body, JsonBody, NdBody};
    use serde_json::{json, Value as JsonValue};

    use super::*;

    #[test]
    fn test_build_bulk_request_body() {
        #[allow(clippy::type_complexity)]
        let tests: Vec<(Vec<BulkItem<JsonValue>>, Vec<JsonBody<JsonValue>>)> = vec![
            (vec![], vec![]),
            (
                vec![BulkItem::new(
                    Operation::Index(OperationParams::default()),
                    json!({}),
                )],
                vec![json!({"index": {}}).into(), json!({}).into()],
            ),
            (
                vec![
                    BulkItem::new(Operation::Index(OperationParams::default()), json!({})),
                    BulkItem::new(Operation::Index(OperationParams::default()), json!({})),
                ],
                vec![
                    json!({"index": {}}).into(),
                    json!({}).into(),
                    json!({"index": {}}).into(),
                    json!({}).into(),
                ],
            ),
            (
                vec![BulkItem::new(
                    Operation::Index(OperationParams::default()),
                    json!({"action":"Login"}),
                )],
                vec![
                    json!({"index": {}}).into(),
                    json!({"action": "Login"}).into(),
                ],
            ),
            (
                vec![BulkItem::new(
                    Operation::Delete(OperationParams::builder()
                        .id("0".into())
                        .build()
                    ),
                    json!({"action":"Login"}),
                )],
                vec![
                    json!({"delete": { "_id": "0" }}).into(),
                    json!({"action": "Login"}).into(),
                ],
            ),
            (
                vec![BulkItem::new(
                    Operation::Create(OperationParams::builder()
                        .index("action-logs-main".into())
                        .build()
                    ),
                    json!({"action":"Login"}),
                )],
                vec![
                    json!({"create": { "_index": "action-logs-main" }}).into(),
                    json!({"action": "Login"}).into(),
                ],
            ),
            (
                vec![
                    BulkItem::new(
                        Operation::Index(OperationParams::builder()
                            .id("0".into())
                            .index("action-logs-main".into())
                            .build()
                        ),
                        json!({"action":"Login"}),
                    ),
                    BulkItem::new(
                        Operation::Update(OperationParams::builder()
                            .id("1".into())
                            .index("action-logs-trading".into())
                            .build()
                        ),
                        json!({"action":"AddOrder"}),
                    ),
                ],
                vec![
                    json!({"index": { "_id": "0", "_index": "action-logs-main" }}).into(),
                    json!({"action": "Login"}).into(),
                    json!({"update": { "_id": "1", "_index": "action-logs-trading" }}).into(),
                    json!({"action": "AddOrder"}).into(),
                ],
            ),
            (
                vec![
                    BulkItem::new(
                        Operation::Index(OperationParams::builder()
                            .id("0".into())
                            .index("nft".into())
                            .version(Version::External(1))
                            .build()
                        ),
                        json!({"action":"Login"}),
                    ),
                    BulkItem::new(
                        Operation::Index(OperationParams::builder()
                            .id("1".into())
                            .index("collection".into())
                            .version(Version::ExternalGte(2))
                            .build()
                        ),
                        json!({"action":"AddOrder"}),
                    ),
                ],
                vec![
                    json!({"index": { "_id": "0", "_index": "nft", "version": 1, "version_type": "external" }}).into(),
                    json!({"action": "Login"}).into(),
                    json!({"index": { "_id": "1", "_index": "collection", "version": 2, "version_type": "external_gte" }}).into(),
                    json!({"action": "AddOrder"}).into(),
                ],
            ),
        ];

        for test in tests {
            let result: Vec<JsonBody<JsonValue>> =
                build_bulk_request_body(test.0)
                    .expect("Failed to construct bulk request body");
            let result = NdBody::new(result);
            let mut result_buf = bytes::BytesMut::new();
            result
                .write(&mut result_buf)
                .expect("Failed to serialize result");

            let expect = NdBody::new(test.1);
            let mut expect_buf = bytes::BytesMut::new();
            expect
                .write(&mut expect_buf)
                .expect("Failed to serialize expected value");

            assert_eq!(result_buf, expect_buf);
        }
    }

    #[tokio::test]
    async fn test_disabled() {
        let config = Config {
            url: "http://localhost:9200".into(),
            enabled: false,
            ..Default::default()
        };
        assert!(matches!(
            ElasticSearch::new(&config).await,
            Err(Error::ElasticSearchDisabled)
        ),);
    }

    #[tokio::test]
    async fn test_ca_not_found() {
        let config = Config {
            url: "https://localhost:9200".into(),
            enabled: true,
            tls: Some(TlsConfig {
                ca: Some("ca-cert-not-found.crt".into()),
                ..Default::default()
            }),
            ..Default::default()
        };
        assert!(matches!(
            ElasticSearch::new(&config).await,
            Err(Error::CertificateError(_, _))
        ));
    }

    #[ignore]
    #[tokio::test]
    async fn test_query() {
        let config = Config {
            url: "http://localhost:9200".into(),
            enabled: true,
            pool_max_size: Some(2),
            ..Default::default()
        };
        let client = ElasticSearch::new(&config)
            .await
            .expect("Failed to configure Elasticsearch connection");
        let conn = client.get_conn();
        let response = conn
            .query(
                Some(&["action-logs"]),
                Option::<&JsonValue>::None,
                Some(json!({
                    "query": {
                        "match": {
                            "user.user_id": "42"
                        }
                    }
                })),
            )
            .await
            .expect("Failed to query Elasticsearch");
        println!("response = {:?}", response);
        tokio::time::sleep(std::time::Duration::from_millis(1)).await;
    }

    #[test]
    fn test_bulk_results_deserialization() {
        let tests = vec![
            (
                json!({
                    "errors": false,
                    "items": [
                        {
                            "index": {
                                "_index": "foo", "_id": "1", "status": 201, "result": "created"
                            }
                        }
                    ]
                }),
                BulkResults {
                    errors: false,
                    results: vec![Operation::Index(OperationStatus {
                        index: Some("foo".into()),
                        id: Some("1".into()),
                        http_code: 201,
                        result: OperationResult::Ok("created".into()),
                    })],
                },
            ),
            (
                json!({
                    "errors": false,
                    "items": [
                        {
                            "create": {
                                "_index": "foo", "_id": "1", "status": 201, "result": "created"
                            }
                        },
                        {
                            "delete": {
                                "_index": "foo", "_id": "2", "status": 404, "result": "not_found"
                            }
                        },
                        {
                            "index": {
                                "_index": "foo", "_id": "3", "status": 201, "result": "created"
                            }
                        },
                        {
                            "update": {
                                "_index": "foo", "status": 201, "result": "updated"
                            }
                        }
                    ]
                }),
                BulkResults {
                    errors: false,
                    results: vec![
                        Operation::Create(OperationStatus {
                            index: Some("foo".into()),
                            id: Some("1".into()),
                            http_code: 201,
                            result: OperationResult::Ok("created".into()),
                        }),
                        Operation::Delete(OperationStatus {
                            index: Some("foo".into()),
                            id: Some("2".into()),
                            http_code: 404,
                            result: OperationResult::Ok("not_found".into()),
                        }),
                        Operation::Index(OperationStatus {
                            index: Some("foo".into()),
                            id: Some("3".into()),
                            http_code: 201,
                            result: OperationResult::Ok("created".into()),
                        }),
                        Operation::Update(OperationStatus {
                            index: Some("foo".into()),
                            id: None,
                            http_code: 201,
                            result: OperationResult::Ok("updated".into()),
                        }),
                    ],
                },
            ),
            (
                json!({
                    "errors": true,
                    "items": [
                        {
                            "index": {
                                "_index": "foo", "_id": "1", "status": 201, "result": "created"
                            }
                        },
                        {
                            "update": {
                                "_index": "foo", "_id": "2", "status": 404, "error": { "type": "document_missing_exception", "reason": "[_doc][2]: document missing" }
                            }
                        }
                    ]
                }),
                BulkResults {
                    errors: true,
                    results: vec![
                        Operation::Index(OperationStatus {
                            index: Some("foo".into()),
                            id: Some("1".into()),
                            http_code: 201,
                            result: OperationResult::Ok("created".into()),
                        }),
                        Operation::Update(OperationStatus {
                            index: Some("foo".into()),
                            id: Some("2".into()),
                            http_code: 404,
                            result: OperationResult::Error {
                                kind: "document_missing_exception".into(),
                                reason: "[_doc][2]: document missing".into(),
                            },
                        }),
                    ],
                },
            ),
        ];

        for test in tests {
            let err = format!("Failed to deserialize: `{}`", test.0);
            let res: BulkResults =
                serde_json::from_value(test.0.clone()).expect(&err);
            assert_eq!(res, test.1, "`{}`", test.0);
        }
    }
}
