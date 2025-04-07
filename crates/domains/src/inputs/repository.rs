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
        let created_at = BlockTimestamp::now();
        let record = sqlx::query_as::<_, InputDbItem>(
            r#"
            INSERT INTO inputs (
                subject,
                value,
                block_height,
                tx_id,
                tx_index,
                input_index,
                cursor,
                type,
                utxo_id,
                amount,
                asset_id,
                owner_id,
                balance_root,
                contract_id,
                state_root,
                sender_address,
                recipient_address,
                nonce,
                data,
                data_length,
                witness_index,
                predicate_gas_used,
                predicate,
                predicate_data,
                predicate_length,
                predicate_data_length,
                block_time,
                created_at
            )
            VALUES (
                $1, $2, $3, $4, $5, $6, $7, $8::input_type, $9, $10,
                $11, $12, $13, $14, $15, $16, $17, $18, $19, $20,
                $21, $22, $23, $24, $25, $26, $27, $28
            )
            ON CONFLICT (subject) DO UPDATE SET
                value = EXCLUDED.value,
                block_height = EXCLUDED.block_height,
                tx_id = EXCLUDED.tx_id,
                tx_index = EXCLUDED.tx_index,
                input_index = EXCLUDED.input_index,
                cursor = EXCLUDED.cursor,
                type = EXCLUDED.type,
                utxo_id = EXCLUDED.utxo_id,
                amount = EXCLUDED.amount,
                asset_id = EXCLUDED.asset_id,
                owner_id = EXCLUDED.owner_id,
                balance_root = EXCLUDED.balance_root,
                contract_id = EXCLUDED.contract_id,
                state_root = EXCLUDED.state_root,
                sender_address = EXCLUDED.sender_address,
                recipient_address = EXCLUDED.recipient_address,
                nonce = EXCLUDED.nonce,
                data = EXCLUDED.data,
                data_length = EXCLUDED.data_length,
                witness_index = EXCLUDED.witness_index,
                predicate_gas_used = EXCLUDED.predicate_gas_used,
                predicate = EXCLUDED.predicate,
                predicate_data = EXCLUDED.predicate_data,
                predicate_length = EXCLUDED.predicate_length,
                predicate_data_length = EXCLUDED.predicate_data_length,
                block_time = EXCLUDED.block_time,
                created_at = EXCLUDED.created_at
            RETURNING *
            "#,
        )
        .bind(&db_item.subject)
        .bind(&db_item.value)
        .bind(db_item.block_height.into_inner() as i64)
        .bind(&db_item.tx_id)
        .bind(db_item.tx_index)
        .bind(db_item.input_index)
        .bind(db_item.cursor().to_string())
        .bind(db_item.r#type)
        .bind(&db_item.utxo_id)
        .bind(db_item.amount)
        .bind(&db_item.asset_id)
        .bind(&db_item.owner_id)
        .bind(&db_item.balance_root)
        .bind(&db_item.contract_id)
        .bind(&db_item.state_root)
        .bind(&db_item.sender_address)
        .bind(&db_item.recipient_address)
        .bind(&db_item.nonce)
        .bind(&db_item.data)
        .bind(db_item.data_length)
        .bind(db_item.witness_index)
        .bind(db_item.predicate_gas_used)
        .bind(&db_item.predicate)
        .bind(&db_item.predicate_data)
        .bind(db_item.predicate_length)
        .bind(db_item.predicate_data_length)
        .bind(db_item.block_time)
        .bind(created_at)
        .fetch_one(executor)
        .await
        .map_err(RepositoryError::Insert)?;

        Ok(record)
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use anyhow::{Ok, Result};
    use fuel_streams_types::BlockHeight;
    use pretty_assertions::assert_eq;

    use super::*;
    use crate::{
        blocks::{
            packets::DynBlockSubject,
            repository::tests::insert_block,
            Block,
            BlockDbItem,
        },
        infra::{
            Db,
            DbConnectionOpts,
            OrderBy,
            QueryOptions,
            QueryParamsBuilder,
            RecordPointer,
        },
        inputs::DynInputSubject,
        mocks::{MockInput, MockTransaction},
        transactions::{
            repository::tests::insert_transaction,
            DynTransactionSubject,
            Transaction,
            TransactionDbItem,
        },
    };

    async fn setup_db() -> anyhow::Result<(Arc<Db>, String)> {
        let db_opts = DbConnectionOpts::default();
        let db = Db::new(db_opts).await?;
        let namespace = QueryOptions::random_namespace();
        Ok((db, namespace))
    }

    fn assert_result(result: &InputDbItem, expected: &InputDbItem) {
        assert_eq!(result.cursor(), expected.cursor());
        assert_eq!(result.subject, expected.subject);
        assert_eq!(result.value, expected.value);
        assert_eq!(result.block_height, expected.block_height);
        assert_eq!(result.tx_id, expected.tx_id);
        assert_eq!(result.tx_index, expected.tx_index);
        assert_eq!(result.input_index, expected.input_index);
        assert_eq!(result.r#type, expected.r#type);
        assert_eq!(result.owner_id, expected.owner_id);
        assert_eq!(result.sender_address, expected.sender_address);
        assert_eq!(result.recipient_address, expected.recipient_address);
        assert_eq!(result.amount, expected.amount);
        assert_eq!(result.asset_id, expected.asset_id);
        assert_eq!(result.balance_root, expected.balance_root);
        assert_eq!(result.contract_id, expected.contract_id);
        assert_eq!(result.state_root, expected.state_root);
        assert_eq!(result.nonce, expected.nonce);
        assert_eq!(result.data, expected.data);
        assert_eq!(result.data_length, expected.data_length);
        assert_eq!(result.witness_index, expected.witness_index);
        assert_eq!(result.predicate_gas_used, expected.predicate_gas_used);
        assert_eq!(result.predicate, expected.predicate);
        assert_eq!(result.predicate_data, expected.predicate_data);
        assert_eq!(result.predicate_length, expected.predicate_length);
        assert_eq!(
            result.predicate_data_length,
            expected.predicate_data_length
        );
    }

    async fn insert_random_block(
        db: &Arc<Db>,
        height: BlockHeight,
        namespace: &str,
    ) -> Result<(BlockDbItem, Block, DynBlockSubject)> {
        let (db_item, block, subject) =
            insert_block(db, height, namespace).await?;
        Ok((db_item, block, subject))
    }

    async fn insert_tx(
        db: &Arc<Db>,
        tx: &Transaction,
        height: BlockHeight,
        namespace: &str,
    ) -> Result<(TransactionDbItem, Transaction, DynTransactionSubject)> {
        let _ = insert_random_block(db, height, namespace).await?;
        Ok(insert_transaction(db, Some(tx.clone()), height, namespace).await?)
    }

    async fn insert_input(
        db: &Arc<Db>,
        tx: &Transaction,
        input: &Input,
        height: BlockHeight,
        namespace: &str,
        (tx_index, input_index): (i32, i32),
    ) -> Result<(InputDbItem, Input, DynInputSubject)> {
        let subject = DynInputSubject::new(
            input,
            height,
            tx.id.to_owned(),
            tx_index,
            input_index,
        );
        let timestamps = BlockTimestamp::default();
        let packet = subject
            .build_packet(input, timestamps, RecordPointer {
                block_height: height,
                tx_id: Some(tx.id.to_owned()),
                tx_index: Some(tx_index as u32),
                input_index: Some(input_index as u32),
                ..Default::default()
            })
            .with_namespace(namespace);

        let db_item = InputDbItem::try_from(&packet)?;
        let result = Input::insert(db.pool_ref(), &db_item).await?;
        assert_result(&result, &db_item);

        Ok((db_item, input.clone(), subject))
    }

    async fn create_inputs(
        db: &Arc<Db>,
        namespace: &str,
        count: u32,
    ) -> Result<Vec<InputDbItem>> {
        let mut inputs = Vec::with_capacity(count as usize);
        for _ in 0..count {
            inputs.push(MockInput::coin_predicate())
        }

        let height = BlockHeight::random();
        let tx = MockTransaction::script(inputs.clone(), vec![], vec![]);
        insert_tx(db, &tx, height, namespace).await?;

        let mut db_items = Vec::with_capacity(count as usize);
        for (index, input) in inputs.iter().enumerate() {
            let (db_item, _, _) = insert_input(
                db,
                &tx,
                input,
                height,
                namespace,
                (0, index as i32),
            )
            .await?;
            db_items.push(db_item);
        }
        db_items.sort_by_key(|i| i.cursor());
        Ok(db_items)
    }

    #[tokio::test]
    async fn test_inserting_input_coin() -> Result<()> {
        let (db, namespace) = setup_db().await?;
        let height = BlockHeight::random();
        let input1 = MockInput::coin_signed(None);
        let input2 = MockInput::coin_predicate();
        let tx = MockTransaction::script(
            vec![input1.clone(), input2.clone()],
            vec![],
            vec![],
        );
        insert_tx(&db, &tx, height, &namespace).await?;
        insert_input(&db, &tx, &input1, height, &namespace, (0, 0)).await?;
        insert_input(&db, &tx, &input2, height, &namespace, (0, 1)).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_inserting_input_contract() -> Result<()> {
        let (db, namespace) = setup_db().await?;
        let height = BlockHeight::random();
        let input = MockInput::contract();
        let tx = MockTransaction::script(vec![input.clone()], vec![], vec![]);
        insert_tx(&db, &tx, height, &namespace).await?;
        insert_input(&db, &tx, &input, height, &namespace, (0, 0)).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_inserting_input_message() -> Result<()> {
        let (db, namespace) = setup_db().await?;
        let height = BlockHeight::random();
        let input1 = MockInput::message_coin_signed();
        let input2 = MockInput::message_coin_predicate();
        let input3 = MockInput::message_data_signed();
        let input4 = MockInput::message_data_predicate();
        let tx = MockTransaction::script(
            vec![
                input1.clone(),
                input2.clone(),
                input3.clone(),
                input4.clone(),
            ],
            vec![],
            vec![],
        );

        insert_tx(&db, &tx, height, &namespace).await?;
        insert_input(&db, &tx, &input1, height, &namespace, (0, 0)).await?;
        insert_input(&db, &tx, &input2, height, &namespace, (0, 1)).await?;
        insert_input(&db, &tx, &input3, height, &namespace, (0, 2)).await?;
        insert_input(&db, &tx, &input4, height, &namespace, (0, 3)).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_find_one_input() -> Result<()> {
        let (db, namespace) = setup_db().await?;
        let height = BlockHeight::random();
        let input = MockInput::coin_signed(None);
        let tx = MockTransaction::script(vec![input.clone()], vec![], vec![]);
        insert_tx(&db, &tx, height, &namespace).await?;

        let (db_item, _, subject) =
            insert_input(&db, &tx, &input, height, &namespace, (0, 0)).await?;
        let mut query = subject.to_query_params();
        query.with_namespace(Some(namespace));

        let result = Input::find_one(db.pool_ref(), &query).await?;
        assert_result(&result, &db_item);

        Ok(())
    }

    #[tokio::test]
    async fn test_find_many_inputs_basic_query() -> Result<()> {
        let (db, namespace) = setup_db().await?;
        let inputs = create_inputs(&db, &namespace, 3).await?;

        let mut query = InputsQuery::default();
        query.with_namespace(Some(namespace));
        query.with_order_by(OrderBy::Asc);

        let results = Input::find_many(db.pool_ref(), &query).await?;
        assert_eq!(results.len(), 3, "Should find all three inputs");
        assert_result(&results[0], &inputs[0]);
        assert_result(&results[1], &inputs[1]);
        assert_result(&results[2], &inputs[2]);

        Ok(())
    }

    #[tokio::test]
    async fn test_find_many_inputs_with_cursor_based_pagination_after(
    ) -> Result<()> {
        let (db, namespace) = setup_db().await?;
        let inputs = create_inputs(&db, &namespace, 5).await?;

        let mut query = InputsQuery::default();
        query.with_namespace(Some(namespace));
        query.with_after(Some(inputs[1].cursor()));
        query.with_first(Some(2));

        let results = Input::find_many(db.pool_ref(), &query).await?;
        assert_eq!(
            results.len(),
            2,
            "Should return exactly 2 inputs after cursor"
        );
        assert_result(&results[0], &inputs[2]);
        assert_result(&results[1], &inputs[3]);

        Ok(())
    }

    #[tokio::test]
    async fn test_find_many_inputs_with_cursor_based_pagination_before(
    ) -> Result<()> {
        let (db, namespace) = setup_db().await?;
        let inputs = create_inputs(&db, &namespace, 5).await?;

        let mut query = InputsQuery::default();
        query.with_namespace(Some(namespace));
        query.with_before(Some(inputs[4].cursor()));
        query.with_last(Some(2));

        let results = Input::find_many(db.pool_ref(), &query).await?;
        assert_eq!(
            results.len(),
            2,
            "Should return exactly 2 inputs before cursor"
        );
        assert_result(&results[0], &inputs[3]);
        assert_result(&results[1], &inputs[2]);

        Ok(())
    }

    #[tokio::test]
    async fn test_find_many_inputs_with_limit_offset_pagination() -> Result<()>
    {
        let (db, namespace) = setup_db().await?;
        let inputs = create_inputs(&db, &namespace, 5).await?;

        let mut query = InputsQuery::default();
        query.with_namespace(Some(namespace));
        query.with_limit(Some(2));
        query.with_offset(Some(1));
        query.with_order_by(OrderBy::Asc);

        let results = Input::find_many(db.pool_ref(), &query).await?;
        assert_eq!(results.len(), 2, "Should return exactly 2 inputs");
        assert_result(&results[0], &inputs[1]);
        assert_result(&results[1], &inputs[2]);

        Ok(())
    }

    #[tokio::test]
    async fn test_find_many_inputs_with_different_order() -> Result<()> {
        let (db, namespace) = setup_db().await?;
        let mut inputs = create_inputs(&db, &namespace, 3).await?;

        let mut query = InputsQuery::default();
        query.with_namespace(Some(namespace));
        query.with_order_by(OrderBy::Asc);

        let asc_results = Input::find_many(db.pool_ref(), &query).await?;
        assert_eq!(asc_results.len(), 3);
        assert_result(&asc_results[0], &inputs[0]);
        assert_result(&asc_results[2], &inputs[2]);

        query.with_order_by(OrderBy::Desc);
        inputs.sort_by_key(|a| a.cursor());
        let desc_results = Input::find_many(db.pool_ref(), &query).await?;
        assert_eq!(desc_results.len(), 3);
        assert_result(&desc_results[0], &inputs[2]);
        assert_result(&desc_results[2], &inputs[0]);

        Ok(())
    }

    #[tokio::test]
    async fn test_cursor_pagination_ignores_order_by() -> Result<()> {
        let (db, namespace) = setup_db().await?;
        let inputs = create_inputs(&db, &namespace, 5).await?;

        let mut query = InputsQuery::default();
        query.with_namespace(Some(namespace));
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
