use async_trait::async_trait;
use fuel_streams_types::BlockTimestamp;
use sqlx::{Acquire, PgExecutor, Postgres};

use super::{Utxo, UtxoDbItem, UtxosQuery};
use crate::infra::{
    repository::{Repository, RepositoryError, RepositoryResult},
    DbItem,
};

#[async_trait]
impl Repository for Utxo {
    type Item = UtxoDbItem;
    type QueryParams = UtxosQuery;

    async fn insert<'e, 'c: 'e, E>(
        executor: E,
        db_item: &Self::Item,
    ) -> RepositoryResult<Self::Item>
    where
        'c: 'e,
        E: PgExecutor<'c> + Acquire<'c, Database = Postgres>,
    {
        let published_at = BlockTimestamp::now();
        let record = sqlx::query_as::<_, UtxoDbItem>(
            "WITH upsert AS (
                INSERT INTO utxos (
                    subject, value, cursor, block_height, tx_id, tx_index,
                    input_index, utxo_type, utxo_id, contract_id, created_at, published_at
                )
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
                ON CONFLICT (subject) DO UPDATE SET
                    value = EXCLUDED.value,
                    cursor = EXCLUDED.cursor,
                    block_height = EXCLUDED.block_height,
                    tx_id = EXCLUDED.tx_id,
                    tx_index = EXCLUDED.tx_index,
                    input_index = EXCLUDED.input_index,
                    utxo_type = EXCLUDED.utxo_type,
                    utxo_id = EXCLUDED.utxo_id,
                    contract_id = EXCLUDED.contract_id,
                    created_at = EXCLUDED.created_at,
                    published_at = $12
                RETURNING *
            )
            SELECT * FROM upsert",
        )
        .bind(&db_item.subject)
        .bind(&db_item.value)
        .bind(db_item.cursor().to_string())
        .bind(db_item.block_height)
        .bind(&db_item.tx_id)
        .bind(db_item.tx_index)
        .bind(db_item.input_index)
        .bind(&db_item.utxo_type)
        .bind(&db_item.utxo_id)
        .bind(&db_item.contract_id)
        .bind(db_item.created_at)
        .bind(published_at)
        .fetch_one(executor)
        .await
        .map_err(RepositoryError::Insert)?;

        Ok(record)
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use fuel_streams_types::primitives::*;
    use pretty_assertions::assert_eq;

    use super::*;
    use crate::{
        infra::{
            Db,
            DbConnectionOpts,
            OrderBy,
            QueryOptions,
            QueryParamsBuilder,
        },
        inputs::Input,
        mocks::{MockInput, MockTransaction},
        utxos::DynUtxoSubject,
    };

    async fn setup_db() -> anyhow::Result<(Arc<Db>, String)> {
        let db_opts = DbConnectionOpts::default();
        let db = Db::new(db_opts).await?;
        let namespace = QueryOptions::random_namespace();
        Ok((db, namespace))
    }

    fn assert_result(result: &UtxoDbItem, expected: &UtxoDbItem) {
        assert_eq!(result.cursor(), expected.cursor());
        assert_eq!(result.subject, expected.subject);
        assert_eq!(result.value, expected.value);
        assert_eq!(result.block_height, expected.block_height);
        assert_eq!(result.tx_id, expected.tx_id);
        assert_eq!(result.tx_index, expected.tx_index);
        assert_eq!(result.input_index, expected.input_index);
        assert_eq!(result.utxo_type, expected.utxo_type);
        assert_eq!(result.utxo_id, expected.utxo_id);
        assert_eq!(result.contract_id, expected.contract_id);
        assert_eq!(result.created_at, expected.created_at);
    }

    async fn insert_utxo(
        db: &Arc<Db>,
        input: Option<Input>,
        height: u32,
        namespace: &str,
    ) -> anyhow::Result<(UtxoDbItem, Input, DynUtxoSubject)> {
        let input = input.unwrap_or_else(MockInput::coin_predicate);
        let tx =
            MockTransaction::script(vec![input.to_owned()], vec![], vec![]);
        let subject = DynUtxoSubject::new(&input, height.into(), tx.id, 0, 0);
        let timestamps = BlockTimestamp::default();
        let packet = subject.build_packet(timestamps).with_namespace(namespace);

        let db_item = UtxoDbItem::try_from(&packet)?;
        let result = Utxo::insert(db.pool_ref(), &db_item).await?;
        assert_result(&result, &db_item);

        Ok((db_item, input, subject))
    }

    async fn create_utxos(
        db: &Arc<Db>,
        namespace: &str,
        count: u32,
    ) -> anyhow::Result<Vec<UtxoDbItem>> {
        let mut utxos = Vec::with_capacity(count as usize);
        for height in 1..=count {
            let (db_item, _, _) =
                insert_utxo(db, None, height, namespace).await?;
            utxos.push(db_item);
        }
        Ok(utxos)
    }

    #[tokio::test]
    async fn test_inserting_coin_utxo() -> anyhow::Result<()> {
        let (db, namespace) = setup_db().await?;
        let input = MockInput::coin_predicate();
        insert_utxo(&db, Some(input), 1, &namespace).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_inserting_contract_utxo() -> anyhow::Result<()> {
        let (db, namespace) = setup_db().await?;
        let input = MockInput::contract();
        insert_utxo(&db, Some(input), 1, &namespace).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_inserting_message_utxo() -> anyhow::Result<()> {
        let (db, namespace) = setup_db().await?;
        let input = MockInput::message_coin_signed();
        insert_utxo(&db, Some(input), 1, &namespace).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_find_one_utxo() -> anyhow::Result<()> {
        let (db, namespace) = setup_db().await?;
        let (db_item, _, subject) =
            insert_utxo(&db, None, 1, &namespace).await?;

        let mut query = subject.to_query_params();
        query.with_namespace(Some(namespace));

        let result = Utxo::find_one(db.pool_ref(), &query).await?;
        assert_result(&result, &db_item);

        Ok(())
    }

    #[tokio::test]
    async fn test_find_many_utxos_basic_query() -> anyhow::Result<()> {
        let (db, namespace) = setup_db().await?;
        let utxos = create_utxos(&db, &namespace, 3).await?;

        let mut query = UtxosQuery::default();
        query.with_namespace(Some(namespace));
        query.with_order_by(OrderBy::Asc);

        let results = Utxo::find_many(db.pool_ref(), &query).await?;
        assert_eq!(results.len(), 3, "Should find all three UTXOs");
        assert_result(&results[0], &utxos[0]);
        assert_result(&results[1], &utxos[1]);
        assert_result(&results[2], &utxos[2]);

        Ok(())
    }

    #[tokio::test]
    async fn test_find_many_utxos_with_cursor_based_pagination_after(
    ) -> anyhow::Result<()> {
        let (db, namespace) = setup_db().await?;
        let utxos = create_utxos(&db, &namespace, 5).await?;

        let mut query = UtxosQuery::default();
        query.with_namespace(Some(namespace));
        query.with_after(Some(utxos[1].cursor()));
        query.with_first(Some(2));

        let results = Utxo::find_many(db.pool_ref(), &query).await?;
        assert_eq!(
            results.len(),
            2,
            "Should return exactly 2 UTXOs after cursor"
        );
        assert_result(&results[0], &utxos[2]);
        assert_result(&results[1], &utxos[3]);

        Ok(())
    }

    #[tokio::test]
    async fn test_find_many_utxos_with_cursor_based_pagination_before(
    ) -> anyhow::Result<()> {
        let (db, namespace) = setup_db().await?;
        let utxos = create_utxos(&db, &namespace, 5).await?;

        let mut query = UtxosQuery::default();
        query.with_namespace(Some(namespace));
        query.with_before(Some(utxos[4].cursor()));
        query.with_last(Some(2));

        let results = Utxo::find_many(db.pool_ref(), &query).await?;
        assert_eq!(
            results.len(),
            2,
            "Should return exactly 2 UTXOs before cursor"
        );
        assert_result(&results[0], &utxos[3]);
        assert_result(&results[1], &utxos[2]);

        Ok(())
    }

    #[tokio::test]
    async fn test_find_many_utxos_with_limit_offset_pagination(
    ) -> anyhow::Result<()> {
        let (db, namespace) = setup_db().await?;
        let utxos = create_utxos(&db, &namespace, 5).await?;

        let mut query = UtxosQuery::default();
        query.with_namespace(Some(namespace));
        query.with_limit(Some(2));
        query.with_offset(Some(1));
        query.with_order_by(OrderBy::Asc);

        let results = Utxo::find_many(db.pool_ref(), &query).await?;
        assert_eq!(results.len(), 2, "Should return exactly 2 UTXOs");
        assert_result(&results[0], &utxos[1]);
        assert_result(&results[1], &utxos[2]);

        Ok(())
    }

    #[tokio::test]
    async fn test_find_many_utxos_with_different_order() -> anyhow::Result<()> {
        let (db, namespace) = setup_db().await?;
        let utxos = create_utxos(&db, &namespace, 3).await?;

        let mut query = UtxosQuery::default();
        query.with_namespace(Some(namespace));
        query.with_order_by(OrderBy::Asc);

        let asc_results = Utxo::find_many(db.pool_ref(), &query).await?;
        assert_eq!(asc_results.len(), 3);
        assert_result(&asc_results[0], &utxos[0]);
        assert_result(&asc_results[2], &utxos[2]);

        query.with_order_by(OrderBy::Desc);
        let desc_results = Utxo::find_many(db.pool_ref(), &query).await?;
        assert_eq!(desc_results.len(), 3);
        assert_result(&desc_results[0], &utxos[2]);
        assert_result(&desc_results[2], &utxos[0]);

        Ok(())
    }

    #[tokio::test]
    async fn test_cursor_pagination_ignores_order_by() -> anyhow::Result<()> {
        let (db, namespace) = setup_db().await?;
        let utxos = create_utxos(&db, &namespace, 5).await?;

        let mut query = UtxosQuery::default();
        query.with_namespace(Some(namespace));
        query.with_after(Some(utxos[1].cursor()));
        query.with_first(Some(2));

        let results_default = Utxo::find_many(db.pool_ref(), &query).await?;
        query.with_order_by(OrderBy::Asc);
        let results_asc = Utxo::find_many(db.pool_ref(), &query).await?;
        query.with_order_by(OrderBy::Desc);
        let results_desc = Utxo::find_many(db.pool_ref(), &query).await?;

        assert_eq!(results_default, results_asc);
        assert_eq!(results_default, results_desc);

        Ok(())
    }
}
