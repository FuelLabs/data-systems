use std::sync::Arc;

use fuel_streams_core::prelude::*;

pub mod cli;

#[derive(Debug, Clone, Default)]
pub enum Client {
    #[default]
    Core,
    Publisher,
}

impl Client {
    pub fn url(&self, cli: &cli::Cli) -> String {
        match self {
            Client::Core => cli.nats_core_url.clone(),
            Client::Publisher => cli.nats_publisher_url.clone(),
        }
    }
    pub async fn new(
        &self,
        cli: &cli::Cli,
    ) -> Result<Arc<NatsClient>, NatsError> {
        let opts = NatsClientOpts::admin_opts(None);
        let opts = opts.with_custom_url(self.url(cli));
        Ok(Arc::new(NatsClient::connect(&opts).await?))
    }
}
