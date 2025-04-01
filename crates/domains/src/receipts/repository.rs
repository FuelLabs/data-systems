use async_trait::async_trait;
use fuel_streams_types::BlockTimestamp;
use sqlx::{Acquire, PgExecutor, Postgres};

use super::{Receipt, ReceiptDbItem, ReceiptsQuery};
use crate::infra::repository::{Repository, RepositoryError, RepositoryResult};

#[async_trait]
impl Repository for Receipt {
    type Item = ReceiptDbItem;
    type QueryParams = ReceiptsQuery;

    async fn insert<'e, 'c: 'e, E>(
        executor: E,
        db_item: &Self::Item,
    ) -> RepositoryResult<Self::Item>
    where
        'c: 'e,
        E: PgExecutor<'c> + Acquire<'c, Database = Postgres>,
    {
        let published_at = BlockTimestamp::now();
        let record = sqlx::query_as::<_, ReceiptDbItem>(
            "WITH upsert AS (
                INSERT INTO receipts (
                    subject, value, block_height, tx_id, tx_index, receipt_index,
                    receipt_type, from_contract_id, to_contract_id, to_address,
                    asset_id, contract_id, sub_id, sender_address, recipient_address,
                    created_at, published_at
                )
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17)
                ON CONFLICT (subject) DO UPDATE SET
                    value = EXCLUDED.value,
                    block_height = EXCLUDED.block_height,
                    tx_id = EXCLUDED.tx_id,
                    tx_index = EXCLUDED.tx_index,
                    receipt_index = EXCLUDED.receipt_index,
                    receipt_type = EXCLUDED.receipt_type,
                    from_contract_id = EXCLUDED.from_contract_id,
                    to_contract_id = EXCLUDED.to_contract_id,
                    to_address = EXCLUDED.to_address,
                    asset_id = EXCLUDED.asset_id,
                    contract_id = EXCLUDED.contract_id,
                    sub_id = EXCLUDED.sub_id,
                    sender_address = EXCLUDED.sender_address,
                    recipient_address = EXCLUDED.recipient_address,
                    created_at = EXCLUDED.created_at,
                    published_at = $17
                RETURNING *
            )
            SELECT * FROM upsert"
        )
        .bind(db_item.subject.clone())
        .bind(db_item.value.to_owned())
        .bind(db_item.block_height)
        .bind(db_item.tx_id.to_owned())
        .bind(db_item.tx_index)
        .bind(db_item.receipt_index)
        .bind(db_item.receipt_type.to_owned())
        .bind(db_item.from_contract_id.to_owned())
        .bind(db_item.to_contract_id.to_owned())
        .bind(db_item.to_address.to_owned())
        .bind(db_item.asset_id.to_owned())
        .bind(db_item.contract_id.to_owned())
        .bind(db_item.sub_id.to_owned())
        .bind(db_item.sender_address.to_owned())
        .bind(db_item.recipient_address.to_owned())
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
    use pretty_assertions::assert_eq;

    use super::*;
    use crate::{
        infra::{Db, DbConnectionOpts, QueryOptions, ToPacket},
        mocks::{MockReceipt, MockTransaction},
        receipts::DynReceiptSubject,
    };

    async fn test_receipt(receipt: &Receipt) -> anyhow::Result<()> {
        let db_opts = DbConnectionOpts::default();
        let db = Db::new(db_opts).await?;
        let tx =
            MockTransaction::script(vec![], vec![], vec![receipt.to_owned()]);
        let namespace = QueryOptions::random_namespace();
        let subject =
            DynReceiptSubject::from((receipt, 1.into(), tx.id.clone(), 0, 0));
        let timestamps = BlockTimestamp::default();
        let packet = receipt
            .to_packet(&subject.into(), timestamps)
            .with_namespace(&namespace);

        let db_item = ReceiptDbItem::try_from(&packet)?;
        let result = Receipt::insert(db.pool_ref(), &db_item).await;
        assert!(result.is_ok());

        let result = result.unwrap();
        assert_eq!(result.subject, db_item.subject);
        assert_eq!(result.value, db_item.value);
        assert_eq!(result.block_height, db_item.block_height);
        assert_eq!(result.tx_id, db_item.tx_id);
        assert_eq!(result.tx_index, db_item.tx_index);
        assert_eq!(result.receipt_index, db_item.receipt_index);
        assert_eq!(result.receipt_type, db_item.receipt_type);
        assert_eq!(result.from_contract_id, db_item.from_contract_id);
        assert_eq!(result.to_contract_id, db_item.to_contract_id);
        assert_eq!(result.to_address, db_item.to_address);
        assert_eq!(result.asset_id, db_item.asset_id);
        assert_eq!(result.contract_id, db_item.contract_id);
        assert_eq!(result.sub_id, db_item.sub_id);
        assert_eq!(result.sender_address, db_item.sender_address);
        assert_eq!(result.recipient_address, db_item.recipient_address);
        assert_eq!(result.created_at, db_item.created_at);

        Ok(())
    }

    #[tokio::test]
    async fn test_inserting_receipt_call() -> anyhow::Result<()> {
        test_receipt(&MockReceipt::call()).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_inserting_receipt_return() -> anyhow::Result<()> {
        test_receipt(&MockReceipt::return_receipt()).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_inserting_receipt_return_data() -> anyhow::Result<()> {
        test_receipt(&MockReceipt::return_data()).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_inserting_receipt_panic() -> anyhow::Result<()> {
        test_receipt(&MockReceipt::panic()).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_inserting_receipt_revert() -> anyhow::Result<()> {
        test_receipt(&MockReceipt::revert()).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_inserting_receipt_log() -> anyhow::Result<()> {
        test_receipt(&MockReceipt::log()).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_inserting_receipt_log_data() -> anyhow::Result<()> {
        test_receipt(&MockReceipt::log_data()).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_inserting_receipt_transfer() -> anyhow::Result<()> {
        test_receipt(&MockReceipt::transfer()).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_inserting_receipt_transfer_out() -> anyhow::Result<()> {
        test_receipt(&MockReceipt::transfer_out()).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_inserting_receipt_script_result() -> anyhow::Result<()> {
        test_receipt(&MockReceipt::script_result()).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_inserting_receipt_message_out() -> anyhow::Result<()> {
        test_receipt(&MockReceipt::message_out()).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_inserting_receipt_mint() -> anyhow::Result<()> {
        test_receipt(&MockReceipt::mint()).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_inserting_receipt_burn() -> anyhow::Result<()> {
        test_receipt(&MockReceipt::burn()).await?;
        Ok(())
    }
}
