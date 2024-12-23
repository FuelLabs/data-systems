use std::sync::Arc;

use fuel_streams_core::prelude::*;

use super::ClientError;

/// A client for connecting to a NATS server.
///
/// This struct represents a connected NATS client.
#[derive(Debug, Clone)]
pub struct Client {
    /// The underlying NATS client connection.
    pub nats_conn: Arc<NatsClient>,
    pub s3_conn: Arc<S3Client>,
}

impl Client {
    /// Connects to a NATS server using the provided URL.
    ///
    /// # Parameters
    ///
    /// * `network`: An enum variant representing the fuel network we are connecting to.
    ///
    /// # Returns
    ///
    /// Returns a `Result` containing the connected client on success, or an error on failure.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use fuel_streams::client::{Client, FuelNetwork};
    ///
    /// # async fn example() -> Result<(), fuel_streams::Error> {
    /// let client = Client::connect(FuelNetwork::Local).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn connect(network: FuelNetwork) -> Result<Self, crate::Error> {
        let nats_opts =
            NatsClientOpts::public_opts().with_url(network.to_nats_url());
        let nats_client = NatsClient::connect(&nats_opts)
            .await
            .map_err(ClientError::NatsConnectionFailed)?;

        let s3_client_opts = match network {
            FuelNetwork::Local => {
                S3ClientOpts::new(S3Env::Local, S3Role::Admin)
            }
            FuelNetwork::Testnet => {
                S3ClientOpts::new(S3Env::Testnet, S3Role::Public)
            }
            FuelNetwork::Mainnet => {
                S3ClientOpts::new(S3Env::Mainnet, S3Role::Public)
            }
        };

        let s3_client = S3Client::new(&s3_client_opts)
            .await
            .map_err(ClientError::S3ConnectionFailed)?;

        Ok(Self {
            nats_conn: Arc::new(nats_client),
            s3_conn: Arc::new(s3_client),
        })
    }

    /// Connects to a NATS server using the provided options.
    ///
    /// # Parameters
    ///
    /// * `opts`: A reference to `NatsClientOpts` containing the connection options.
    ///
    /// # Returns
    ///
    /// Returns a `ConnectionResult` containing the connected client on success, or an error on failure.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use fuel_streams::client::{Client, FuelNetwork};
    /// use fuel_streams_core::nats::NatsClientOpts;
    /// use fuel_streams_core::s3::{S3ClientOpts, S3Env, S3Role};
    ///
    /// # async fn example() -> Result<(), fuel_streams::Error> {
    /// let nats_opts = NatsClientOpts::public_opts().with_url("nats://localhost:4222");
    /// let s3_opts = S3ClientOpts::new(S3Env::Local, S3Role::Admin);
    ///
    /// let client = Client::with_opts(&nats_opts, &s3_opts).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn with_opts(
        nats_opts: &NatsClientOpts,
        s3_opts: &S3ClientOpts,
    ) -> Result<Self, crate::Error> {
        let nats_client = NatsClient::connect(nats_opts)
            .await
            .map_err(ClientError::NatsConnectionFailed)?;
        let s3_client = S3Client::new(s3_opts)
            .await
            .map_err(ClientError::S3ConnectionFailed)?;
        Ok(Self {
            nats_conn: Arc::new(nats_client),
            s3_conn: Arc::new(s3_client),
        })
    }
}
