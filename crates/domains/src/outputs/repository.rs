use async_trait::async_trait;
use fuel_streams_types::BlockTimestamp;
use sqlx::{Acquire, PgExecutor, Postgres};

use super::{Output, OutputDbItem, OutputsQuery};
use crate::infra::{
    repository::{Repository, RepositoryError, RepositoryResult},
    DbItem,
};

#[async_trait]
impl Repository for Output {
    type Item = OutputDbItem;
    type QueryParams = OutputsQuery;

    async fn insert<'e, 'c: 'e, E>(
        executor: E,
        db_item: &Self::Item,
    ) -> RepositoryResult<Self::Item>
    where
        'c: 'e,
        E: PgExecutor<'c> + Acquire<'c, Database = Postgres>,
    {
        let published_at = BlockTimestamp::now();
        let record = sqlx::query_as::<_, OutputDbItem>(
            "WITH upsert AS (
                INSERT INTO outputs (
                    subject, value, cursor, block_height, tx_id, tx_index,
                    output_index, output_type, to_address, asset_id, contract_id,
                    created_at, published_at
                )
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)
                ON CONFLICT (subject) DO UPDATE SET
                    value = EXCLUDED.value,
                    cursor = EXCLUDED.cursor,
                    block_height = EXCLUDED.block_height,
                    tx_id = EXCLUDED.tx_id,
                    tx_index = EXCLUDED.tx_index,
                    output_index = EXCLUDED.output_index,
                    output_type = EXCLUDED.output_type,
                    to_address = EXCLUDED.to_address,
                    asset_id = EXCLUDED.asset_id,
                    contract_id = EXCLUDED.contract_id,
                    created_at = EXCLUDED.created_at,
                    published_at = $13
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
        .bind(db_item.output_index)
        .bind(&db_item.output_type)
        .bind(&db_item.to_address)
        .bind(&db_item.asset_id)
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

    use anyhow::Result;
    use pretty_assertions::assert_eq;

    use super::*;
    use crate::{
        infra::{
            Db,
            DbConnectionOpts,
            DbItem,
            OrderBy,
            QueryOptions,
            QueryParamsBuilder,
        },
        mocks::{MockOutput, MockTransaction},
        outputs::DynOutputSubject,
    };

    async fn setup_db() -> anyhow::Result<(Arc<Db>, String)> {
        let db_opts = DbConnectionOpts::default();
        let db = Db::new(db_opts).await?;
        let namespace = QueryOptions::random_namespace();
        Ok((db, namespace))
    }

    fn assert_result(result: &OutputDbItem, expected: &OutputDbItem) {
        assert_eq!(result.cursor(), expected.cursor());
        assert_eq!(result.subject, expected.subject);
        assert_eq!(result.value, expected.value);
        assert_eq!(result.block_height, expected.block_height);
        assert_eq!(result.tx_id, expected.tx_id);
        assert_eq!(result.tx_index, expected.tx_index);
        assert_eq!(result.output_index, expected.output_index);
        assert_eq!(result.output_type, expected.output_type);
        assert_eq!(result.to_address, expected.to_address);
        assert_eq!(result.asset_id, expected.asset_id);
        assert_eq!(result.contract_id, expected.contract_id);
        assert_eq!(result.created_at, expected.created_at);
    }

    async fn insert_output(
        db: &Arc<Db>,
        output: Option<Output>,
        height: u32,
        namespace: &str,
    ) -> Result<(OutputDbItem, Output, DynOutputSubject)> {
        let output = output.unwrap_or_else(|| MockOutput::coin(100));
        let tx =
            MockTransaction::script(vec![], vec![output.to_owned()], vec![]);

        let subject = DynOutputSubject::new(
            &output,
            height.into(),
            tx.id.clone(),
            0,
            0,
            &tx,
        );
        let timestamps = BlockTimestamp::default();
        let packet = subject
            .build_packet(&output, timestamps)
            .with_namespace(namespace);

        let db_item = OutputDbItem::try_from(&packet)?;
        let result = Output::insert(db.pool_ref(), &db_item).await?;
        assert_result(&result, &db_item);

        Ok((db_item, output, subject))
    }

    async fn create_outputs(
        db: &Arc<Db>,
        namespace: &str,
        count: u32,
    ) -> Result<Vec<OutputDbItem>> {
        let mut outputs = Vec::with_capacity(count as usize);
        for height in 1..=count {
            let (db_item, _, _) =
                insert_output(db, None, height, namespace).await?;
            outputs.push(db_item);
        }
        Ok(outputs)
    }

    #[tokio::test]
    async fn test_inserting_output_coin() -> Result<()> {
        let (db, namespace) = setup_db().await?;
        insert_output(&db, Some(MockOutput::coin(100)), 1, &namespace).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_inserting_output_contract() -> Result<()> {
        let (db, namespace) = setup_db().await?;
        insert_output(&db, Some(MockOutput::contract()), 1, &namespace).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_inserting_output_change() -> Result<()> {
        let (db, namespace) = setup_db().await?;
        insert_output(&db, Some(MockOutput::change(50)), 1, &namespace).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_inserting_output_variable() -> Result<()> {
        let (db, namespace) = setup_db().await?;
        insert_output(&db, Some(MockOutput::variable(75)), 1, &namespace)
            .await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_inserting_output_contract_created() -> Result<()> {
        let (db, namespace) = setup_db().await?;
        insert_output(&db, Some(MockOutput::contract_created()), 1, &namespace)
            .await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_find_one_output() -> Result<()> {
        let (db, namespace) = setup_db().await?;
        let (db_item, _, subject) =
            insert_output(&db, None, 1, &namespace).await?;

        let mut query = subject.to_query_params();
        query.with_namespace(Some(namespace));

        let result = Output::find_one(db.pool_ref(), &query).await?;
        assert_result(&result, &db_item);

        Ok(())
    }

    #[tokio::test]
    async fn test_find_many_outputs_basic_query() -> Result<()> {
        let (db, namespace) = setup_db().await?;
        let outputs = create_outputs(&db, &namespace, 3).await?;

        let mut query = OutputsQuery::default();
        query.with_namespace(Some(namespace));
        query.with_order_by(OrderBy::Asc);

        let results = Output::find_many(db.pool_ref(), &query).await?;
        assert_eq!(results.len(), 3, "Should find all three outputs");
        assert_result(&results[0], &outputs[0]);
        assert_result(&results[1], &outputs[1]);
        assert_result(&results[2], &outputs[2]);

        Ok(())
    }

    #[tokio::test]
    async fn test_find_many_outputs_with_cursor_based_pagination_after(
    ) -> Result<()> {
        let (db, namespace) = setup_db().await?;
        let outputs = create_outputs(&db, &namespace, 5).await?;

        let mut query = OutputsQuery::default();
        query.with_namespace(Some(namespace));
        query.with_after(Some(outputs[1].cursor()));
        query.with_first(Some(2));

        let results = Output::find_many(db.pool_ref(), &query).await?;
        assert_eq!(
            results.len(),
            2,
            "Should return exactly 2 outputs after cursor"
        );
        assert_result(&results[0], &outputs[2]);
        assert_result(&results[1], &outputs[3]);

        Ok(())
    }

    #[tokio::test]
    async fn test_find_many_outputs_with_cursor_based_pagination_before(
    ) -> Result<()> {
        let (db, namespace) = setup_db().await?;
        let outputs = create_outputs(&db, &namespace, 5).await?;

        let mut query = OutputsQuery::default();
        query.with_namespace(Some(namespace));
        query.with_before(Some(outputs[4].cursor()));
        query.with_last(Some(2));

        let results = Output::find_many(db.pool_ref(), &query).await?;
        assert_eq!(
            results.len(),
            2,
            "Should return exactly 2 outputs before cursor"
        );
        assert_result(&results[0], &outputs[3]);
        assert_result(&results[1], &outputs[2]);

        Ok(())
    }

    #[tokio::test]
    async fn test_find_many_outputs_with_limit_offset_pagination() -> Result<()>
    {
        let (db, namespace) = setup_db().await?;
        let outputs = create_outputs(&db, &namespace, 5).await?;

        let mut query = OutputsQuery::default();
        query.with_namespace(Some(namespace));
        query.with_limit(Some(2));
        query.with_offset(Some(1));
        query.with_order_by(OrderBy::Asc);

        let results = Output::find_many(db.pool_ref(), &query).await?;
        assert_eq!(results.len(), 2, "Should return exactly 2 outputs");
        assert_result(&results[0], &outputs[1]);
        assert_result(&results[1], &outputs[2]);

        Ok(())
    }

    #[tokio::test]
    async fn test_find_many_outputs_with_different_order() -> Result<()> {
        let (db, namespace) = setup_db().await?;
        let outputs = create_outputs(&db, &namespace, 3).await?;

        let mut query = OutputsQuery::default();
        query.with_namespace(Some(namespace));
        query.with_order_by(OrderBy::Asc);

        let asc_results = Output::find_many(db.pool_ref(), &query).await?;
        assert_eq!(asc_results.len(), 3);
        assert_result(&asc_results[0], &outputs[0]);
        assert_result(&asc_results[2], &outputs[2]);

        query.with_order_by(OrderBy::Desc);
        let desc_results = Output::find_many(db.pool_ref(), &query).await?;
        assert_eq!(desc_results.len(), 3);
        assert_result(&desc_results[0], &outputs[2]);
        assert_result(&desc_results[2], &outputs[0]);

        Ok(())
    }

    #[tokio::test]
    async fn test_cursor_pagination_ignores_order_by() -> Result<()> {
        let (db, namespace) = setup_db().await?;
        let outputs = create_outputs(&db, &namespace, 5).await?;

        let mut query = OutputsQuery::default();
        query.with_namespace(Some(namespace));
        query.with_after(Some(outputs[1].cursor()));
        query.with_first(Some(2));

        let results_default = Output::find_many(db.pool_ref(), &query).await?;
        query.with_order_by(OrderBy::Asc);
        let results_asc = Output::find_many(db.pool_ref(), &query).await?;
        query.with_order_by(OrderBy::Desc);
        let results_desc = Output::find_many(db.pool_ref(), &query).await?;

        assert_eq!(results_default, results_asc);
        assert_eq!(results_default, results_desc);

        Ok(())
    }
}
