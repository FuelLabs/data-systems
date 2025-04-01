use fuel_streams_core::types::Block;
use fuel_streams_domains::infra::{db::Db, QueryOptions};
use fuel_streams_types::BlockHeight;

use crate::error::PublishError;

#[derive(Debug, Clone)]
pub struct BlockHeightGap {
    pub start: BlockHeight,
    pub end: BlockHeight,
}

pub async fn find_next_block_to_save(
    db: &Db,
    fuel_core_height: BlockHeight,
) -> Result<Vec<BlockHeightGap>, PublishError> {
    let select = r#"
    WITH block_sequence AS (
        SELECT
            block_height,
            LAG(block_height) OVER (ORDER BY block_height) AS prev_block
        FROM blocks
    )
    SELECT
        prev_block + 1 AS gap_start,
        block_height - 1 AS gap_end
    FROM block_sequence
    WHERE block_height > prev_block + 1;
    "#;

    let gaps = sqlx::query_as::<_, (i64, i64)>(select)
        .fetch_all(&db.pool)
        .await
        .map_err(PublishError::from)?
        .into_iter()
        .map(|(start, end)| BlockHeightGap {
            start: start.into(),
            end: end.into(),
        })
        .collect::<Vec<_>>();

    if gaps.is_empty() {
        // If no gaps found, get the last saved block height and create a gap from there
        let last_height =
            Block::find_last_block_height(db, &QueryOptions::default()).await?;
        return Ok(vec![BlockHeightGap {
            start: ((*last_height) + 1).into(),
            end: fuel_core_height,
        }]);
    }

    Ok(gaps)
}
