use async_trait::async_trait;
use fuel_streams_types::BlockTimestamp;
use sqlx::{Acquire, PgExecutor, Postgres};

use super::{Input, InputDbItem, InputsQuery};
use crate::infra::{
    repository::{Repository, RepositoryError, RepositoryResult},
    DbItem,
};

#[async_trait]
impl Repository for Input {
    type Item = InputDbItem;
    type QueryParams = InputsQuery;

    async fn insert<'e, 'c: 'e, E>(
        executor: E,
        db_item: &Self::Item,
    ) -> RepositoryResult<Self::Item>
    where
        'c: 'e,
        E: PgExecutor<'c> + Acquire<'c, Database = Postgres>,
    {
        let published_at = BlockTimestamp::now();
        let record = sqlx::query_as::<_, InputDbItem>(
            "WITH upsert AS (
                INSERT INTO inputs (
                    subject, value, cursor, block_height, tx_id, tx_index,
                    input_index, input_type, owner_id, asset_id,
                    contract_id, sender_address, recipient_address,
                    created_at, published_at
                )
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15)
                ON CONFLICT (subject) DO UPDATE SET
                    value = EXCLUDED.value,
                    cursor = EXCLUDED.cursor,
                    block_height = EXCLUDED.block_height,
                    tx_id = EXCLUDED.tx_id,
                    tx_index = EXCLUDED.tx_index,
                    input_index = EXCLUDED.input_index,
                    input_type = EXCLUDED.input_type,
                    owner_id = EXCLUDED.owner_id,
                    asset_id = EXCLUDED.asset_id,
                    contract_id = EXCLUDED.contract_id,
                    sender_address = EXCLUDED.sender_address,
                    recipient_address = EXCLUDED.recipient_address,
                    created_at = EXCLUDED.created_at,
                    published_at = $15
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
        .bind(&db_item.input_type)
        .bind(&db_item.owner_id)
        .bind(&db_item.asset_id)
        .bind(&db_item.contract_id)
        .bind(&db_item.sender_address)
        .bind(&db_item.recipient_address)
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
        inputs::DynInputSubject,
        mocks::{MockInput, MockTransaction},
    };

    async fn test_input(input: &Input) -> Result<()> {
        let db_opts = DbConnectionOpts::default();
        let db = Db::new(db_opts).await?;
        let tx =
            MockTransaction::script(vec![input.to_owned()], vec![], vec![]);
        let namespace = QueryOptions::random_namespace();
        let subject =
            DynInputSubject::new(input, 1.into(), tx.id.clone(), 0, 0);
        let timestamps = BlockTimestamp::default();
        let packet = subject
            .build_packet(input, timestamps)
            .with_namespace(&namespace);

        let db_item = InputDbItem::try_from(&packet)?;
        let result = Input::insert(db.pool_ref(), &db_item).await;
        assert!(result.is_ok());

        let result = result.unwrap();
        assert_eq!(db_item.cursor(), result.cursor());
        assert_eq!(result.subject, db_item.subject);
        assert_eq!(result.value, db_item.value);
        assert_eq!(result.block_height, db_item.block_height);
        assert_eq!(result.tx_id, db_item.tx_id);
        assert_eq!(result.tx_index, db_item.tx_index);
        assert_eq!(result.input_index, db_item.input_index);
        assert_eq!(result.input_type, db_item.input_type);
        assert_eq!(result.owner_id, db_item.owner_id);
        assert_eq!(result.asset_id, db_item.asset_id);
        assert_eq!(result.contract_id, db_item.contract_id);
        assert_eq!(result.sender_address, db_item.sender_address);
        assert_eq!(result.recipient_address, db_item.recipient_address);

        Ok(())
    }

    async fn create_test_input(
        height: u32,
        namespace: &str,
    ) -> (InputDbItem, Input, DynInputSubject) {
        let input = MockInput::coin_signed();
        let tx =
            MockTransaction::script(vec![input.to_owned()], vec![], vec![]);
        let subject =
            DynInputSubject::new(&input, height.into(), tx.id.clone(), 0, 0);
        let timestamps = BlockTimestamp::default();
        let packet = subject
            .build_packet(&input, timestamps)
            .with_namespace(namespace);
        let db_item = InputDbItem::try_from(&packet).unwrap();
        (db_item, input, subject)
    }

    async fn create_inputs(
        namespace: &str,
        db: &Db,
        count: u32,
    ) -> Vec<InputDbItem> {
        let mut inputs = Vec::with_capacity(count as usize);
        for height in 1..=count {
            let (db_item, _, _) = create_test_input(height, namespace).await;
            Input::insert(db.pool_ref(), &db_item).await.unwrap();
            inputs.push(db_item);
        }
        inputs
    }

    #[tokio::test]
    async fn test_inserting_input_coin() -> Result<()> {
        test_input(&MockInput::coin_signed()).await?;
        test_input(&MockInput::coin_predicate()).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_inserting_input_contract() -> Result<()> {
        test_input(&MockInput::contract()).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_inserting_input_message() -> Result<()> {
        test_input(&MockInput::message_coin_signed()).await?;
        test_input(&MockInput::message_coin_predicate()).await?;
        test_input(&MockInput::message_data_signed()).await?;
        test_input(&MockInput::message_data_predicate()).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_find_one_input() -> Result<()> {
        let namespace = QueryOptions::random_namespace();
        let db_opts = DbConnectionOpts::default();
        let db = Db::new(db_opts).await?;
        let (db_item, _, subject) = create_test_input(1, &namespace).await;
        let mut query = subject.to_query_params();
        query.with_namespace(Some(namespace));

        Input::insert(db.pool_ref(), &db_item).await?;
        let result = Input::find_one(db.pool_ref(), &query).await?;
        assert_eq!(result.subject, db_item.subject);
        assert_eq!(result.value, db_item.value);
        assert_eq!(result.block_height, db_item.block_height);
        assert_eq!(result.tx_id, db_item.tx_id);
        assert_eq!(result.input_type, db_item.input_type);

        Ok(())
    }

    #[tokio::test]
    async fn test_find_many_inputs_basic_query() -> Result<()> {
        let namespace = QueryOptions::random_namespace();
        let db_opts = DbConnectionOpts::default();
        let db = Db::new(db_opts).await?;
        let inputs = create_inputs(&namespace, &db, 3).await;
        let mut query = InputsQuery::default();
        query.with_namespace(Some(namespace));
        query.with_order_by(OrderBy::Asc);

        let results = Input::find_many(db.pool_ref(), &query).await?;
        assert_eq!(results.len(), 3, "Should find all three inputs");
        assert_eq!(results[0].subject, inputs[0].subject);
        assert_eq!(results[1].subject, inputs[1].subject);
        assert_eq!(results[2].subject, inputs[2].subject);

        Ok(())
    }

    #[tokio::test]
    async fn test_find_many_inputs_with_cursor_based_pagination_after(
    ) -> Result<()> {
        let namespace = QueryOptions::random_namespace();
        let db_opts = DbConnectionOpts::default();
        let db = Db::new(db_opts).await?;
        let inputs = create_inputs(&namespace, &db, 5).await;

        let mut query = InputsQuery::default();
        query.with_namespace(Some(namespace.clone()));
        query.with_after(Some(inputs[1].cursor()));
        query.with_first(Some(2));

        let results = Input::find_many(db.pool_ref(), &query).await?;
        assert_eq!(
            results.len(),
            2,
            "Should return exactly 2 inputs after cursor"
        );
        assert_eq!(results[0].cursor(), inputs[2].cursor());
        assert_eq!(results[1].cursor(), inputs[3].cursor());

        Ok(())
    }

    #[tokio::test]
    async fn test_find_many_inputs_with_cursor_based_pagination_before(
    ) -> Result<()> {
        let namespace = QueryOptions::random_namespace();
        let db_opts = DbConnectionOpts::default();
        let db = Db::new(db_opts).await?;
        let inputs = create_inputs(&namespace, &db, 5).await;
        let mut query = InputsQuery::default();
        query.with_namespace(Some(namespace.clone()));
        query.with_before(Some(inputs[4].cursor()));
        query.with_last(Some(2));

        let results = Input::find_many(db.pool_ref(), &query).await?;
        assert_eq!(
            results.len(),
            2,
            "Should return exactly 2 inputs before cursor"
        );
        assert_eq!(results[0].cursor(), inputs[3].cursor());
        assert_eq!(results[1].cursor(), inputs[2].cursor());

        Ok(())
    }

    #[tokio::test]
    async fn test_find_many_inputs_with_limit_offset_pagination() -> Result<()>
    {
        let namespace = QueryOptions::random_namespace();
        let db_opts = DbConnectionOpts::default();
        let db = Db::new(db_opts).await?;
        let inputs = create_inputs(&namespace, &db, 5).await;

        // Test first page
        let mut query = InputsQuery::default();
        query.with_namespace(Some(namespace.clone()));
        query.with_limit(Some(2));
        query.with_offset(Some(0));
        query.with_order_by(OrderBy::Asc);

        let first_page = Input::find_many(db.pool_ref(), &query).await?;
        assert_eq!(first_page.len(), 2, "First page should have 2 inputs");
        assert_eq!(first_page[0].cursor(), inputs[0].cursor());
        assert_eq!(first_page[1].cursor(), inputs[1].cursor());

        // Test second page
        query.with_offset(Some(2));
        let second_page = Input::find_many(db.pool_ref(), &query).await?;
        assert_eq!(second_page.len(), 2, "Second page should have 2 inputs");
        assert_eq!(second_page[0].cursor(), inputs[2].cursor());
        assert_eq!(second_page[1].cursor(), inputs[3].cursor());

        // Test last page
        query.with_offset(Some(4));
        let last_page = Input::find_many(db.pool_ref(), &query).await?;
        assert_eq!(last_page.len(), 1, "Last page should have 1 input");
        assert_eq!(last_page[0].cursor(), inputs[4].cursor());

        Ok(())
    }

    #[tokio::test]
    async fn test_find_many_inputs_with_different_order() -> Result<()> {
        let namespace = QueryOptions::random_namespace();
        let db_opts = DbConnectionOpts::default();
        let db = Db::new(db_opts).await?;
        let inputs = create_inputs(&namespace, &db, 3).await;

        // Test ascending order
        let mut query = InputsQuery::default();
        query.with_namespace(Some(namespace.clone()));
        query.with_order_by(OrderBy::Asc);

        let asc_results = Input::find_many(db.pool_ref(), &query).await?;
        assert_eq!(asc_results.len(), 3);
        assert_eq!(asc_results[0].cursor(), inputs[0].cursor());
        assert_eq!(asc_results[2].cursor(), inputs[2].cursor());

        // Test descending order
        query.with_order_by(OrderBy::Desc);
        let desc_results = Input::find_many(db.pool_ref(), &query).await?;
        assert_eq!(desc_results.len(), 3);
        assert_eq!(desc_results[0].cursor(), inputs[2].cursor());
        assert_eq!(desc_results[2].cursor(), inputs[0].cursor());

        Ok(())
    }

    #[tokio::test]
    async fn test_cursor_pagination_ignores_order_by() -> Result<()> {
        let namespace = QueryOptions::random_namespace();
        let db_opts = DbConnectionOpts::default();
        let db = Db::new(db_opts).await?;
        let inputs = create_inputs(&namespace, &db, 5).await;

        let mut query = InputsQuery::default();
        query.with_namespace(Some(namespace.clone()));
        query.with_after(Some(inputs[1].cursor()));
        query.with_first(Some(2));

        let results_default = Input::find_many(db.pool_ref(), &query).await?;
        query.with_order_by(OrderBy::Asc);

        let results_asc = Input::find_many(db.pool_ref(), &query).await?;
        query.with_order_by(OrderBy::Desc);

        let results_desc = Input::find_many(db.pool_ref(), &query).await?;
        assert_eq!(results_default, results_asc);
        assert_eq!(results_default, results_desc);

        Ok(())
    }
}
