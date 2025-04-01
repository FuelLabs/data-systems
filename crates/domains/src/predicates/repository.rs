use async_trait::async_trait;
use fuel_streams_types::BlockTimestamp;
use sqlx::{Acquire, PgExecutor, Postgres};

use super::{Predicate, PredicateDbItem, PredicatesQuery};
use crate::infra::repository::{Repository, RepositoryError, RepositoryResult};

#[async_trait]
impl Repository for Predicate {
    type Item = PredicateDbItem;
    type QueryParams = PredicatesQuery;

    async fn insert<'e, 'c: 'e, E>(
        executor: E,
        db_item: &Self::Item,
    ) -> RepositoryResult<Self::Item>
    where
        'c: 'e,
        E: PgExecutor<'c> + Acquire<'c, Database = Postgres>,
    {
        let published_at = BlockTimestamp::now();
        let record = sqlx::query_as::<_, PredicateDbItem>(
            r#"
            WITH inserted_predicate AS (
                INSERT INTO predicates (
                    blob_id,
                    predicate_address,
                    created_at,
                    published_at
                )
                VALUES ($1, $2, $3, $4)
                ON CONFLICT (predicate_address) DO UPDATE
                SET blob_id = EXCLUDED.blob_id,
                    created_at = EXCLUDED.created_at,
                    published_at = EXCLUDED.published_at
                RETURNING id, blob_id, predicate_address, created_at, published_at
            ),
            inserted_transaction AS (
                INSERT INTO predicate_transactions (
                    predicate_id,
                    subject,
                    block_height,
                    tx_id,
                    tx_index,
                    input_index,
                    asset_id,
                    bytecode
                )
                SELECT
                    id,
                    $5,
                    $6,
                    $7,
                    $8,
                    $9,
                    $10,
                    $11
                FROM inserted_predicate
                RETURNING predicate_id
            )
            SELECT
                p.id,
                p.blob_id,
                p.predicate_address,
                p.created_at,
                p.published_at,
                $5 AS subject,
                $6 AS block_height,
                $7 AS tx_id,
                $8 AS tx_index,
                $9 AS input_index,
                $10 AS asset_id,
                $11 AS bytecode
            FROM inserted_predicate p
            "#,
        )
        .bind(&db_item.blob_id)
        .bind(&db_item.predicate_address)
        .bind(db_item.created_at)
        .bind(published_at)
        .bind(&db_item.subject)
        .bind(db_item.block_height)
        .bind(&db_item.tx_id)
        .bind(db_item.tx_index)
        .bind(db_item.input_index)
        .bind(&db_item.asset_id)
        .bind(&db_item.bytecode)
        .fetch_one(executor)
        .await
        .map_err(|e| {
            eprintln!("SQL error inserting predicate: {:?}", e);
            RepositoryError::Insert(e)
        })?;

        Ok(record)
    }
}

#[cfg(test)]
mod tests {
    use fuel_streams_types::primitives::*;
    use pretty_assertions::assert_eq;

    use super::*;
    use crate::{
        infra::{Db, DbConnectionOpts, QueryOptions, ToPacket},
        inputs::Input,
        mocks::{MockInput, MockTransaction},
        predicates::DynPredicateSubject,
    };

    async fn test_predicate(input: &Input) -> anyhow::Result<()> {
        let db_opts = DbConnectionOpts::default();
        let db = Db::new(db_opts).await?;
        let tx =
            MockTransaction::script(vec![input.to_owned()], vec![], vec![]);
        let namespace = QueryOptions::random_namespace();

        // Create predicate subject
        let dyn_subject =
            DynPredicateSubject::new(input, &1.into(), &tx.id, 0, 0)
                .expect("Failed to create predicate subject");

        let predicate = dyn_subject.predicate().to_owned();
        let timestamps = BlockTimestamp::default();
        let packet = predicate
            .to_packet(&dyn_subject.into(), timestamps)
            .with_namespace(&namespace);

        let db_item = PredicateDbItem::try_from(&packet)?;
        let result = Predicate::insert(db.pool_ref(), &db_item).await;
        assert!(result.is_ok());

        let result = result.unwrap();
        assert_eq!(result.subject, db_item.subject);
        assert_eq!(result.block_height, db_item.block_height);
        assert_eq!(result.tx_id, db_item.tx_id);
        assert_eq!(result.tx_index, db_item.tx_index);
        assert_eq!(result.input_index, db_item.input_index);
        assert_eq!(result.blob_id, db_item.blob_id);
        assert_eq!(result.predicate_address, db_item.predicate_address);
        assert_eq!(result.asset_id, db_item.asset_id);
        assert_eq!(result.bytecode, db_item.bytecode);
        assert_eq!(result.created_at, db_item.created_at);

        Ok(())
    }

    #[tokio::test]
    async fn test_inserting_predicate_with_blob_id() -> anyhow::Result<()> {
        let input = MockInput::coin_predicate();
        test_predicate(&input).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_inserting_predicate_without_blob_id() -> anyhow::Result<()> {
        let input = MockInput::coin_signed();
        test_predicate(&input).await?;
        Ok(())
    }
}
