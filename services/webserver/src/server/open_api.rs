use fuel_streams_domains::{
    blocks::queryable::BlocksQuery,
    transactions::queryable::TransactionsQuery,
    utxos::queryable::UtxosQuery,
};
use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(
    // paths(get_blocks, get_transactions),
    components(schemas(UtxosQuery, BlocksQuery, TransactionsQuery)),
    tags(
        (name = "blockchain-api", description = "Blockchain data endpoints")
    )
)]
pub struct ApiDoc;
