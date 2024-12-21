// TODO: Consider using external lib for elasticsearch
// TODO: Consider modularizing this module further

use std::{fs, io, path::PathBuf, sync::Arc};

use anyhow::Context;
use chrono::Utc;
use displaydoc::Display;
pub use elasticsearch::params::Refresh;
use elasticsearch::{
    self,
    auth::{ClientCertificate, Credentials},
    cert::{Certificate, CertificateValidation},
    http::transport::{SingleNodeConnectionPool, Transport, TransportBuilder},
    params,
    Elasticsearch,
    IndexParts,
};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use url::{self, Url};

pub const ELASTICSEARCH_PATH: &str = "fuel-data-systems";

/// LogEntry represents a log entry that will be stored in Elastic Search
/// for monitoring purposes.
/// TODO: Consider adding more useful optional fields to this struct
#[derive(Serialize, Deserialize)]
pub struct LogEntry {
    timestamp: chrono::DateTime<Utc>,
    level: String,
    message: String,
}

impl LogEntry {
    pub fn new(level: &str, message: &str) -> Self {
        Self {
            timestamp: Utc::now(),
            level: level.to_string(),
            message: message.to_string(),
        }
    }
}

pub async fn log(elastic_search: Arc<ElasticSearch>, log_entry: LogEntry) {
    if let Err(err) = elastic_search
        .get_conn()
        .index(
            ELASTICSEARCH_PATH,
            Some("publisher-logs"),
            &log_entry,
            Some(Refresh::WaitFor),
        )
        .await
    {
        tracing::error!("Failed to log to ElasticSearch: {}", err);
    }
}

pub fn should_use_elasticsearch() -> bool {
    dotenvy::var("USE_ELASTIC_LOGGING").is_ok_and(|val| val == "true")
}

pub async fn new_elastic_search() -> anyhow::Result<ElasticSearch> {
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
pub enum ElasticSearchError {
    /// ElasticSearchConfigError: `{0}`
    Config(#[from] elasticsearch::http::transport::BuildError),
    /// ElasticSearchDisabled
    Disabled,
    /// ElasticSearchError: `{0}`
    Generic(#[from] elasticsearch::Error),
    /// IoError: `{0}`
    Io(#[from] io::Error),
    /// UrlParseError: `{0}`
    UrlParse(#[from] url::ParseError),
    /// CertificateError: `{0}`: `{0}`
    Certificate(PathBuf, io::Error),
    /// SerdeJsonError: `{0}`
    SerdeJson(#[from] serde_json::Error),
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

#[derive(Debug, Clone)]
pub struct ElasticSearch(ElasticConnection);

impl ElasticSearch {
    pub async fn new(config: &Config) -> Result<Self, ElasticSearchError> {
        if !config.enabled {
            return Err(ElasticSearchError::Disabled);
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

#[derive(Clone, Debug)]
pub struct ConnectionInfo(Transport);

impl ConnectionInfo {
    pub fn new(config: &Config) -> Result<Self, ElasticSearchError> {
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
                        ElasticSearchError::Certificate(certificate, err)
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
        let transport = if let Some(ca) = tls.ca {
            transport.cert_validation(CertificateValidation::Full(
                Certificate::from_pem(&fs::read(&ca).map_err(|err| {
                    ElasticSearchError::Certificate(ca.clone(), err)
                })?)
                .map_err(|err| {
                    ElasticSearchError::Certificate(
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

    pub fn get_connection(
        &self,
    ) -> Result<ElasticConnection, ElasticSearchError> {
        let conn = Elasticsearch::new(self.0.clone());
        Ok(ElasticConnection(Some(conn)))
    }
}

#[derive(Debug, Clone)]
pub struct ElasticConnection(Option<Elasticsearch>);

impl ElasticConnection {
    pub fn check_alive(&self) -> Option<bool> {
        Some(self.0.is_some())
    }

    pub async fn ping(&self) -> Result<(), ElasticSearchError> {
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
    ) -> Result<(), ElasticSearchError>
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
}
