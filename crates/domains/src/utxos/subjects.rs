use fuel_streams_subject::subject::*;
use fuel_streams_types::*;
use serde::{Deserialize, Serialize};
use sqlx::{Postgres, QueryBuilder};

use super::types::*;
use crate::infra::{record::QueryOptions, repository::SubjectQueryBuilder};

#[derive(Subject, Debug, Clone, Default, Serialize, Deserialize)]
#[subject(id = "utxos")]
#[subject(entity = "Utxo")]
#[subject(query_all = "utxos.>")]
#[subject(
    format = "utxos.{block_height}.{tx_id}.{tx_index}.{input_index}.{utxo_type}.{utxo_id}.{contract_id}"
)]
pub struct UtxosSubject {
    #[subject(description = "The height of the block containing this UTXO")]
    pub block_height: Option<BlockHeight>,
    #[subject(
        description = "The ID of the transaction containing this UTXO (32 byte string prefixed by 0x)"
    )]
    pub tx_id: Option<TxId>,
    #[subject(description = "The index of the transaction within the block")]
    pub tx_index: Option<u32>,
    #[subject(description = "The index of the input within the transaction")]
    pub input_index: Option<u32>,
    #[subject(description = "The type of UTXO (coin, message, or contract)")]
    pub utxo_type: Option<UtxoType>,
    #[subject(
        description = "The unique identifier for this UTXO (32 byte string prefixed by 0x)"
    )]
    pub utxo_id: Option<HexData>,
    #[subject(
        sql_column = "contract_id",
        description = "The ID of the contract that returned (32 byte string prefixed by 0x)"
    )]
    pub contract_id: Option<ContractId>,
}
impl SubjectQueryBuilder for UtxosSubject {
    fn query_builder(
        &self,
        options: Option<&QueryOptions>,
    ) -> QueryBuilder<'static, Postgres> {
        let mut conditions = Vec::new();
        let mut query_builder: QueryBuilder<Postgres> = QueryBuilder::default();
        query_builder.push("SELECT * FROM utxos");

        if let Some(where_clause) = self.to_sql_where() {
            conditions.push(where_clause);
        }
        if let Some(block) = options.map(|o| o.from_block.unwrap_or_default()) {
            conditions.push(format!("block_height >= {}", block));
        }

        if !conditions.is_empty() {
            query_builder.push(" WHERE ");
            query_builder.push(conditions.join(" AND "));
        }

        query_builder
            .push(" ORDER BY block_height ASC, tx_index ASC, input_index ASC");
        if let Some(opts) = options {
            opts.apply_limit_offset(&mut query_builder);
        }

        query_builder
    }
}
