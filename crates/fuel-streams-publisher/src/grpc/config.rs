use std::{
    path::{Path, PathBuf},
    str::FromStr,
    time::Duration,
};

use clap::Parser;
use confy::ConfyError;
use displaydoc::Display as DisplayDoc;
use serde::{Deserialize, Deserializer, Serialize};
use thiserror::Error;
use tokio::{fs::File, io::AsyncReadExt};

#[derive(Debug, DisplayDoc, Error)]
pub enum Error {
    /// Open config file: {0}
    OpenConfig(std::io::Error),
    /// Failed to parse config: {0}
    ParseConfig(toml::de::Error),
    /// Failed to parse config as utf-8: {0}
    ParseUtf8(std::string::FromUtf8Error),
    /// Failed to read config file: {0}
    ReadConfig(std::io::Error),
    /// Failed to read config metadata: {0}
    ReadMeta(std::io::Error),
    /// Failed to read env config: {0}
    Confy(ConfyError),
    /// Missing config element: {0}
    MissingConfigElement(&'static str),
}

#[derive(Debug, Default, Deserialize, Clone)]
#[serde(deny_unknown_fields, rename_all = "kebab-case")]
pub struct GrpcConfig {
    /// whether to enable gRPC reflection
    pub enable_reflection: bool,
    /// which compression encodings does the server accept for requests
    #[serde(skip_serializing_if = "Option::is_none")]
    pub accept_compressed: Option<String>,
    /// which compression encodings might the server use for responses
    #[serde(skip_serializing_if = "Option::is_none")]
    pub send_compressed: Option<String>,
    /// limits the maximum size of a decoded message. Defaults to 4MB
    pub max_decoding_message_size: usize,
    /// limits the maximum size of an encoded message. Defaults to 4MB
    pub max_encoding_message_size: usize,
    /// limits the maximum size of streaming channel
    #[allow(dead_code)]
    pub max_channel_size: usize,
    /// set a timeout on for all request handlers
    #[serde(deserialize_with = "deserialize_duration_from_usize")]
    pub timeout: Duration,
    /// sets the SETTINGS_INITIAL_WINDOW_SIZE spec option for HTTP2 stream-level flow control.
    /// Default is 65,535
    #[serde(skip_serializing_if = "Option::is_none")]
    pub initial_stream_window_size: Option<u32>,
    /// set whether TCP keepalive messages are enabled on accepted connections
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tcp_keepalive: Option<Duration>,
    /// sets the max connection-level flow control for HTTP2. Default is 65,535
    #[serde(skip_serializing_if = "Option::is_none")]
    pub initial_connection_window_size: Option<u32>,
    /// sets the maximum frame size to use for HTTP2. If not set, will default from underlying
    /// transport
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_frame_size: Option<u32>,
    /// set the concurrency limit applied to on requests inbound per connection. Defaults to 32
    pub concurrency_limit_per_connection: usize,
    /// sets the SETTINGS_MAX_CONCURRENT_STREAMS spec option for HTTP2 connections. Default is no
    /// limit (`None`)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_concurrent_streams: Option<u32>,
    /// set whether HTTP2 Ping frames are enabled on accepted connections. Default is no HTTP2
    /// keepalive (`None`)
    #[serde(
        skip_serializing_if = "Option::is_none",
        deserialize_with = "deserialize_duration_option"
    )]
    pub http2_keepalive_interval: Option<Duration>,
    /// sets a timeout for receiving an acknowledgement of the keepalive ping. Default is 20
    /// seconds
    #[serde(
        skip_serializing_if = "Option::is_none",
        deserialize_with = "deserialize_duration_option"
    )]
    pub http2_keepalive_timeout: Option<Duration>,
    /// sets whether to use an adaptive flow control. Defaults to false
    #[serde(skip_serializing_if = "Option::is_none")]
    pub http2_adaptive_window: Option<bool>,
    /// set the value of `TCP_NODELAY` option for accepted connections. Enabled by default
    pub tcp_nodelay: bool,
    /// when looking for next draw we want to look at max `draw_lookahead_period_count`\
    #[allow(dead_code)]
    pub draw_lookahead_period_count: u64,
}

fn deserialize_duration_from_usize<'de, D>(
    deserializer: D,
) -> Result<Duration, D::Error>
where
    D: Deserializer<'de>,
{
    let seconds = u64::deserialize(deserializer)?;
    Ok(Duration::from_secs(seconds))
}

fn deserialize_duration_option<'de, D>(
    deserializer: D,
) -> Result<Option<Duration>, D::Error>
where
    D: Deserializer<'de>,
{
    let seconds: Option<u64> = Option::deserialize(deserializer)?;
    if seconds.is_none() {
        return Ok(None);
    }
    Ok(seconds.map(Duration::from_secs))
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "kebab-case")]
pub struct TomlConfig {
    pub grpc: GrpcConfig,
}

impl TomlConfig {
    pub async fn new(path: impl AsRef<Path> + Send) -> Result<Self, Error> {
        read_to_string(path).await?.parse()
    }
}

impl FromStr for TomlConfig {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        toml::from_str(s).map_err(Error::ParseConfig)
    }
}

async fn read_to_string(
    path: impl AsRef<Path> + Send,
) -> Result<String, Error> {
    let mut file = File::open(path).await.map_err(Error::OpenConfig)?;
    let meta = file.metadata().await.map_err(Error::ReadMeta)?;
    let mut contents =
        Vec::with_capacity(usize::try_from(meta.len()).unwrap_or(0));
    file.read_to_end(&mut contents)
        .await
        .map_err(Error::ReadConfig)?;
    String::from_utf8(contents).map_err(Error::ParseUtf8)
}

// Cli args and config

#[derive(Clone, Debug, Parser, Default, Deserialize, Serialize)]
pub(crate) struct CliConfig {
    /// toml configuration path
    #[arg(long)]
    toml: PathBuf,
    /// http port
    #[arg(long)]
    http_port: u16,
}

#[allow(dead_code)]
#[derive(Clone, Debug)]
pub(crate) struct Config {
    pub(crate) toml: PathBuf,
}

#[allow(dead_code)]
pub(crate) fn load_config() -> Result<Config, Error> {
    // First parse from cli
    let cli_config = CliConfig::parse();
    // Initialize settings from file if specified
    let file_config = confy::load_path::<CliConfig>(&cli_config.toml)
        .map_err(Error::Confy)?;
    tracing::info!("Loaded grpc config from file: {:?}", &cli_config.toml);

    let config = Config {
        toml: file_config.toml,
    };

    Ok(config)
}
