use fuel_streams_subject::subject::*;
use fuel_streams_types::*;
use serde::{Deserialize, Serialize};
use sqlx::{Postgres, QueryBuilder};

use super::OutputType;
use crate::infra::{record::QueryOptions, repository::SubjectQueryBuilder};

#[derive(Subject, Debug, Clone, Default, Serialize, Deserialize)]
#[subject(id = "outputs_coin")]
#[subject(entity = "Output")]
#[subject(query_all = "outputs.coin.>")]
#[subject(custom_where = "output_type = 'coin'")]
#[subject(
    format = "outputs.coin.{block_height}.{tx_id}.{tx_index}.{output_index}.{to}.{asset}"
)]
pub struct OutputsCoinSubject {
    #[subject(
        description = "The height of the block containing this coin output"
    )]
    pub block_height: Option<BlockHeight>,
    #[subject(
        description = "The ID of the transaction containing this coin output (32 byte string prefixed by 0x)"
    )]
    pub tx_id: Option<TxId>,
    #[subject(description = "The index of the transaction within the block")]
    pub tx_index: Option<u32>,
    #[subject(description = "The index of this output within the transaction")]
    pub output_index: Option<u32>,
    #[subject(
        sql_column = "to_address",
        description = "The recipient address of the coin output (32 byte string prefixed by 0x)"
    )]
    pub to: Option<Address>,
    #[subject(
        sql_column = "asset_id",
        description = "The asset ID of the coin (32 byte string prefixed by 0x)"
    )]
    pub asset: Option<AssetId>,
}

#[derive(Subject, Debug, Clone, Default, Serialize, Deserialize)]
#[subject(id = "outputs_contract")]
#[subject(entity = "Output")]
#[subject(query_all = "outputs.contract.>")]
#[subject(custom_where = "output_type = 'contract'")]
#[subject(
    format = "outputs.contract.{block_height}.{tx_id}.{tx_index}.{output_index}.{contract}"
)]
pub struct OutputsContractSubject {
    #[subject(
        description = "The height of the block containing this contract output"
    )]
    pub block_height: Option<BlockHeight>,
    #[subject(
        description = "The ID of the transaction containing this contract output (32 byte string prefixed by 0x)"
    )]
    pub tx_id: Option<TxId>,
    #[subject(description = "The index of the transaction within the block")]
    pub tx_index: Option<u32>,
    #[subject(description = "The index of this output within the transaction")]
    pub output_index: Option<u32>,
    #[subject(
        sql_column = "contract_id",
        description = "The ID of the contract (32 byte string prefixed by 0x)"
    )]
    pub contract: Option<ContractId>,
}

#[derive(Subject, Debug, Clone, Default, Serialize, Deserialize)]
#[subject(id = "outputs_change")]
#[subject(entity = "Output")]
#[subject(query_all = "outputs.change.>")]
#[subject(custom_where = "output_type = 'change'")]
#[subject(
    format = "outputs.change.{block_height}.{tx_id}.{tx_index}.{output_index}.{to}.{asset}"
)]
pub struct OutputsChangeSubject {
    #[subject(
        description = "The height of the block containing this change output"
    )]
    pub block_height: Option<BlockHeight>,
    #[subject(
        description = "The ID of the transaction containing this change output (32 byte string prefixed by 0x)"
    )]
    pub tx_id: Option<TxId>,
    #[subject(description = "The index of the transaction within the block")]
    pub tx_index: Option<u32>,
    #[subject(description = "The index of this output within the transaction")]
    pub output_index: Option<u32>,
    #[subject(
        sql_column = "to_address",
        description = "The recipient address of the change output (32 byte string prefixed by 0x)"
    )]
    pub to: Option<Address>,
    #[subject(
        sql_column = "asset_id",
        description = "The asset ID of the change output (32 byte string prefixed by 0x)"
    )]
    pub asset: Option<AssetId>,
}

#[derive(Subject, Debug, Clone, Default, Serialize, Deserialize)]
#[subject(id = "outputs_variable")]
#[subject(entity = "Output")]
#[subject(query_all = "outputs.variable.>")]
#[subject(custom_where = "output_type = 'variable'")]
#[subject(
    format = "outputs.variable.{block_height}.{tx_id}.{tx_index}.{output_index}.{to}.{asset}"
)]
pub struct OutputsVariableSubject {
    #[subject(
        description = "The height of the block containing this variable output"
    )]
    pub block_height: Option<BlockHeight>,
    #[subject(
        description = "The ID of the transaction containing this variable output (32 byte string prefixed by 0x)"
    )]
    pub tx_id: Option<TxId>,
    #[subject(description = "The index of the transaction within the block")]
    pub tx_index: Option<u32>,
    #[subject(description = "The index of this output within the transaction")]
    pub output_index: Option<u32>,
    #[subject(
        sql_column = "to_address",
        description = "The recipient address of the variable output (32 byte string prefixed by 0x)"
    )]
    pub to: Option<Address>,
    #[subject(
        sql_column = "asset_id",
        description = "The asset ID of the variable output (32 byte string prefixed by 0x)"
    )]
    pub asset: Option<AssetId>,
}

#[derive(Subject, Debug, Clone, Default, Serialize, Deserialize)]
#[subject(id = "outputs_contract_created")]
#[subject(entity = "Output")]
#[subject(query_all = "outputs.contract_created.>")]
#[subject(custom_where = "output_type = 'contract_created'")]
#[subject(
    format = "outputs.contract_created.{block_height}.{tx_id}.{tx_index}.{output_index}.{contract}"
)]
pub struct OutputsContractCreatedSubject {
    #[subject(
        description = "The height of the block containing this contract creation output"
    )]
    pub block_height: Option<BlockHeight>,
    #[subject(
        description = "The ID of the transaction containing this contract creation output (32 byte string prefixed by 0x)"
    )]
    pub tx_id: Option<TxId>,
    #[subject(description = "The index of the transaction within the block")]
    pub tx_index: Option<u32>,
    #[subject(description = "The index of this output within the transaction")]
    pub output_index: Option<u32>,
    #[subject(
        sql_column = "contract_id",
        description = "The ID of the created contract (32 byte string prefixed by 0x)"
    )]
    pub contract: Option<ContractId>,
}

// This subject is used just for query purpose, not for inserting as key
#[derive(Subject, Debug, Clone, Default, Serialize, Deserialize)]
#[subject(id = "outputs")]
#[subject(entity = "Output")]
#[subject(query_all = "outputs.>")]
#[subject(
    format = "outputs.{output_type}.{block_height}.{tx_id}.{tx_index}.{output_index}"
)]
pub struct OutputsSubject {
    #[subject(
        description = "The type of output (coin, contract, change, variable, or contract_created)"
    )]
    pub output_type: Option<OutputType>,
    #[subject(description = "The height of the block containing this output")]
    pub block_height: Option<BlockHeight>,
    #[subject(
        description = "The ID of the transaction containing this output (32 byte string prefixed by 0x)"
    )]
    pub tx_id: Option<TxId>,
    #[subject(description = "The index of the transaction within the block")]
    pub tx_index: Option<u32>,
    #[subject(description = "The index of this output within the transaction")]
    pub output_index: Option<u32>,
}

fn build_outputs_query(
    where_clause: Option<String>,
    options: Option<&QueryOptions>,
) -> QueryBuilder<'static, Postgres> {
    let mut conditions = Vec::new();
    let mut query_builder: QueryBuilder<Postgres> = QueryBuilder::default();
    query_builder.push("SELECT * FROM outputs");

    if let Some(clause) = where_clause {
        conditions.push(clause);
    }
    if let Some(block) = options.map(|o| o.from_block.unwrap_or_default()) {
        conditions.push(format!("block_height >= {}", block));
    }

    if !conditions.is_empty() {
        query_builder.push(" WHERE ");
        query_builder.push(conditions.join(" AND "));
    }

    query_builder
        .push(" ORDER BY block_height ASC, tx_index ASC, output_index ASC");
    if let Some(opts) = options {
        opts.apply_limit_offset(&mut query_builder);
    }

    query_builder
}

impl SubjectQueryBuilder for OutputsCoinSubject {
    fn query_builder(
        &self,
        options: Option<&QueryOptions>,
    ) -> QueryBuilder<'static, Postgres> {
        build_outputs_query(self.to_sql_where(), options)
    }
}

impl SubjectQueryBuilder for OutputsContractSubject {
    fn query_builder(
        &self,
        options: Option<&QueryOptions>,
    ) -> QueryBuilder<'static, Postgres> {
        build_outputs_query(self.to_sql_where(), options)
    }
}

impl SubjectQueryBuilder for OutputsChangeSubject {
    fn query_builder(
        &self,
        options: Option<&QueryOptions>,
    ) -> QueryBuilder<'static, Postgres> {
        build_outputs_query(self.to_sql_where(), options)
    }
}

impl SubjectQueryBuilder for OutputsVariableSubject {
    fn query_builder(
        &self,
        options: Option<&QueryOptions>,
    ) -> QueryBuilder<'static, Postgres> {
        build_outputs_query(self.to_sql_where(), options)
    }
}

impl SubjectQueryBuilder for OutputsContractCreatedSubject {
    fn query_builder(
        &self,
        options: Option<&QueryOptions>,
    ) -> QueryBuilder<'static, Postgres> {
        build_outputs_query(self.to_sql_where(), options)
    }
}

impl SubjectQueryBuilder for OutputsSubject {
    fn query_builder(
        &self,
        options: Option<&QueryOptions>,
    ) -> QueryBuilder<'static, Postgres> {
        build_outputs_query(self.to_sql_where(), options)
    }
}
