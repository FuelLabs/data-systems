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
        let url = self.url(cli);
        let opts = NatsClientOpts::admin_opts(None)
            .with_custom_url(url)
            .with_domain("CORE".to_string());
        Ok(Arc::new(NatsClient::connect(&opts).await?))
    }
}
