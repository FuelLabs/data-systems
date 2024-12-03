use fuel_streams_core::prelude::*;

use super::ClientError;

/// A client for connecting to a NATS server.
///
/// This struct represents a connected NATS client.
#[derive(Debug, Clone)]
pub struct Client {
    /// The underlying NATS client connection.
    pub conn: NatsClient,
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
        let opts = NatsClientOpts::new(Some(network));
        let conn = NatsClient::connect(&opts)
            .await
            .map_err(ClientError::ConnectionFailed)?;
        Ok(Self { conn })
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
    ///
    /// # async fn example() -> Result<(), fuel_streams::Error> {
    /// let opts = NatsClientOpts::new(FuelNetwork::Local);
    /// let client = Client::with_opts(&opts).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn with_opts(
        opts: &NatsClientOpts,
    ) -> Result<Self, crate::Error> {
        let conn = NatsClient::connect(opts)
            .await
            .map_err(ClientError::ConnectionFailed)?;
        Ok(Self { conn })
    }
}
