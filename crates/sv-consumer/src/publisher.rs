use std::{
    env,
    sync::{Arc, LazyLock},
};

use fuel_streams_core::prelude::*;
use sv_emitter::PublishOpts as EmitterPublishOpts;
use tokio::{sync::Semaphore, task::JoinHandle};

pub static PUBLISHER_MAX_THREADS: LazyLock<usize> = LazyLock::new(|| {
    let available_cpus = num_cpus::get();
    let default_threads = (available_cpus / 3).max(1); // Use 1/3 of CPUs, minimum 1

    env::var("PUBLISHER_MAX_THREADS")
        .ok()
        .and_then(|val| val.parse().ok())
        .unwrap_or(default_threads)
});

#[derive(Debug, Clone)]
pub struct PublishOpts {
    pub chain_id: Arc<FuelCoreChainId>,
    pub base_asset_id: Arc<FuelCoreAssetId>,
    pub block_producer: Arc<Address>,
    pub block_height: Arc<BlockHeight>,
    pub consensus: Arc<Consensus>,
    pub semaphore: Arc<Semaphore>,
}

impl From<EmitterPublishOpts> for PublishOpts {
    fn from(opts: EmitterPublishOpts) -> Self {
        let semaphore = Arc::new(Semaphore::new(1));
        Self {
            chain_id: Arc::new(opts.chain_id),
            base_asset_id: Arc::new(opts.base_asset_id),
            block_producer: Arc::new(opts.block_producer),
            block_height: Arc::new(opts.block_height),
            consensus: Arc::new(opts.consensus),
            semaphore,
        }
    }
}

pub fn publish<S: Streamable + 'static>(
    packet: &PublishPacket<S>,
    stream: Arc<Stream<S>>,
    opts: &Arc<PublishOpts>,
) -> JoinHandle<anyhow::Result<()>> {
    let opts = Arc::clone(opts);
    let payload = Arc::clone(&packet.payload);
    let subject = Arc::clone(&packet.subject);
    let wildcard = packet.subject.parse();

    tokio::spawn(async move {
        let _permit = opts.semaphore.acquire().await?;

        match stream.publish(&*subject, &payload).await {
            Ok(_) => {
                tracing::info!("Successfully published for stream: {wildcard}");
                Ok(())
            }
            Err(e) => {
                anyhow::bail!("Failed to publish: {}", e.to_string())
            }
        }
    })
}
