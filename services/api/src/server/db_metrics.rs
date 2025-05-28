use fuel_streams_domains::blocks::Block;
use fuel_streams_types::BlockHeight;
use tokio::time::{sleep, Duration};

use crate::server::state::ServerState;

pub async fn spawn_block_height_monitor(state: &ServerState) {
    let db = state.db.clone();
    let telemetry = state.telemetry.clone();
    tokio::spawn(async move {
        let mut last_height = BlockHeight::from(0);
        loop {
            if let Ok(new_height) =
                Block::find_last_block_height(&db, &Default::default()).await
            {
                if new_height > last_height {
                    if let Some(metrics) = telemetry.base_metrics() {
                        metrics.update_latest_block_height(
                            new_height.into_inner(),
                        );
                    }
                    last_height = new_height;
                }
            }
            sleep(Duration::from_secs(10)).await;
        }
    });
}
