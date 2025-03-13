use std::sync::Arc;

use fuel_data_parser::DataEncoder;
use fuel_streams_subject::subject::IntoSubject;
use fuel_streams_types::BlockHeight;

use super::StoreError;
use crate::{
    db::Db,
    record::{DbTransaction, QueryOptions, Record},
};

pub type StoreResult<T> = Result<T, StoreError>;

#[derive(Debug, Clone)]
pub struct Store<S: Record + DataEncoder> {
    pub db: Arc<Db>,
    pub namespace: Option<String>,
    _marker: std::marker::PhantomData<S>,
}

impl<R: Record + DataEncoder> Store<R> {
    pub fn new(db: &Arc<Db>) -> Self {
        Self {
            db: Arc::clone(db),
            namespace: None,
            _marker: std::marker::PhantomData,
        }
    }

    pub fn arc(&self) -> Arc<Self> {
        Arc::new(self.to_owned())
    }

    pub fn with_namespace(&mut self, namespace: &str) -> &mut Self {
        self.namespace = Some(namespace.to_string());
        self
    }

    pub async fn insert_record(
        &self,
        db_item: &R::DbItem,
    ) -> StoreResult<R::DbItem> {
        let record = R::insert(&self.db.pool, db_item.to_owned()).await?;
        Ok(record)
    }

    pub async fn insert_record_with_transaction(
        &self,
        tx: &mut DbTransaction,
        db_item: &R::DbItem,
    ) -> StoreResult<R::DbItem> {
        let record = R::insert_with_transaction(tx, db_item).await?;
        Ok(record)
    }

    pub async fn find_many_by_subject(
        &self,
        subject: &Arc<dyn IntoSubject>,
        mut options: QueryOptions,
    ) -> StoreResult<Vec<R::DbItem>> {
        options = options.with_namespace(self.namespace.clone());
        let mut query =
            R::build_find_many_query(subject.clone(), options.clone());
        query
            .build_query_as::<R::DbItem>()
            .fetch_all(&self.db.pool)
            .await
            .map_err(StoreError::from)
    }
}

pub async fn find_last_block_height(
    db: &Db,
    options: QueryOptions,
) -> StoreResult<BlockHeight> {
    let select = "SELECT block_height FROM blocks".to_string();
    let mut query_builder = sqlx::QueryBuilder::new(select);
    if let Some(ns) = options.namespace {
        query_builder
            .push(" WHERE subject LIKE ")
            .push_bind(format!("{}%", ns));
    }

    query_builder.push(" ORDER BY block_height DESC LIMIT 1");
    let query = query_builder.build_query_as::<(i64,)>();
    let record: Option<(i64,)> = query
        .fetch_optional(&db.pool)
        .await
        .map_err(StoreError::FindLastBlockHeight)?;

    Ok(record.map(|(height,)| height.into()).unwrap_or_default())
}

#[derive(Debug, Clone)]
pub struct BlockHeightGap {
    pub start: BlockHeight,
    pub end: BlockHeight,
}

pub async fn find_next_block_to_save(
    db: &Db,
    fuel_core_height: BlockHeight,
) -> StoreResult<Vec<BlockHeightGap>> {
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
        .map_err(StoreError::from)?
        .into_iter()
        .map(|(start, end)| BlockHeightGap {
            start: start.into(),
            end: end.into(),
        })
        .collect::<Vec<_>>();

    if gaps.is_empty() {
        // If no gaps found, get the last saved block height and create a gap from there
        let last_height =
            find_last_block_height(db, QueryOptions::default()).await?;
        return Ok(vec![BlockHeightGap {
            start: ((*last_height) + 1).into(),
            end: fuel_core_height,
        }]);
    }

    Ok(gaps)
}

pub async fn update_block_propagation_ms(
    tx: &mut DbTransaction,
    block_height: BlockHeight,
    propagation_ms: u64,
) -> StoreResult<()> {
    sqlx::query(
        "UPDATE blocks SET block_propagation_ms = $1 WHERE block_height = $2",
    )
    .bind(propagation_ms as i64)
    .bind(block_height)
    .execute(&mut **tx)
    .await
    .map_err(StoreError::from)?;
    Ok(())
}
