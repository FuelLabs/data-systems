use async_trait::async_trait;
use fuel_streams_types::BlockTimestamp;
use sqlx::{Acquire, PgExecutor, Postgres};

use super::{Utxo, UtxoDbItem, UtxosQuery};
use crate::infra::repository::{Repository, RepositoryError, RepositoryResult};

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
                    subject, value, block_height, tx_id, tx_index,
                    input_index, utxo_type, utxo_id, contract_id, created_at, published_at
                )
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
                ON CONFLICT (subject) DO UPDATE SET
                    value = EXCLUDED.value,
                    block_height = EXCLUDED.block_height,
                    tx_id = EXCLUDED.tx_id,
                    tx_index = EXCLUDED.tx_index,
                    input_index = EXCLUDED.input_index,
                    utxo_type = EXCLUDED.utxo_type,
                    utxo_id = EXCLUDED.utxo_id,
                    contract_id = EXCLUDED.contract_id,
                    created_at = EXCLUDED.created_at,
                    published_at = $11
                RETURNING *
            )
            SELECT * FROM upsert",
        )
        .bind(db_item.subject.clone())
        .bind(db_item.value.to_owned())
        .bind(db_item.block_height)
        .bind(db_item.tx_id.to_owned())
        .bind(db_item.tx_index)
        .bind(db_item.input_index)
        .bind(db_item.utxo_type.to_owned())
        .bind(db_item.utxo_id.to_owned())
        .bind(db_item.contract_id.to_owned())
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
    use fuel_streams_types::primitives::*;
    use pretty_assertions::assert_eq;

    use super::*;
    use crate::{
        infra::{Db, DbConnectionOpts, QueryOptions, ToPacket},
        inputs::Input,
        mocks::{MockInput, MockTransaction},
        utxos::DynUtxoSubject,
    };

    async fn test_utxo(input: &Input) -> anyhow::Result<()> {
        let db_opts = DbConnectionOpts::default();
        let db = Db::new(db_opts).await?;
        let tx =
            MockTransaction::script(vec![input.to_owned()], vec![], vec![]);
        let namespace = QueryOptions::random_namespace();

        let subject =
            DynUtxoSubject::from((input, 1.into(), tx.id.clone(), 0, 0));

        let timestamps = BlockTimestamp::default();
        let packet = subject
            .utxo()
            .to_packet(subject.subject(), timestamps)
            .with_namespace(&namespace);

        let db_item = UtxoDbItem::try_from(&packet)?;
        let result = Utxo::insert(db.pool_ref(), &db_item).await;
        assert!(result.is_ok());

        let result = result.unwrap();
        assert_eq!(result.subject, db_item.subject);
        assert_eq!(result.value, db_item.value);
        assert_eq!(result.block_height, db_item.block_height);
        assert_eq!(result.tx_id, db_item.tx_id);
        assert_eq!(result.tx_index, db_item.tx_index);
        assert_eq!(result.input_index, db_item.input_index);
        assert_eq!(result.utxo_type, db_item.utxo_type);
        assert_eq!(result.utxo_id, db_item.utxo_id);
        assert_eq!(result.contract_id, db_item.contract_id);
        assert_eq!(result.created_at, db_item.created_at);

        Ok(())
    }

    #[tokio::test]
    async fn test_inserting_coin_utxo() -> anyhow::Result<()> {
        let input = MockInput::coin_predicate();
        test_utxo(&input).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_inserting_contract_utxo() -> anyhow::Result<()> {
        let input = MockInput::contract();
        test_utxo(&input).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_inserting_message_utxo() -> anyhow::Result<()> {
        let input = MockInput::message_coin_signed();
        test_utxo(&input).await?;
        Ok(())
    }
}
