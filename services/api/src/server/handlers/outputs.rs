use axum::{
    extract::{FromRequest, State},
    http::Request,
    response::IntoResponse,
    Json,
};
use fuel_streams_core::types::*;
use fuel_streams_domains::{
    infra::{
        repository::{Repository, ValidatedQuery},
        Cursor,
        OrderBy,
        TimeRange,
    },
    outputs::OutputsQuery,
};
use paste::paste;

use super::open_api::TAG_OUTPUTS;
use crate::server::{
    errors::ApiError,
    routes::GetDataResponse,
    state::ServerState,
};

#[macro_export]
macro_rules! generate_output_endpoints {
    (
        $tag:expr,
        $base_path:expr,
        $base_name:ident,
        $(
            $variant:ident => $path_suffix:literal
        ),*
    ) => {
        paste! {
            $(
                #[utoipa::path(
                    get,
                    path = concat!($base_path, $path_suffix),
                    tag = $tag,
                    params(
                        ("tx_id" = Option<TxId>, Query, description = "Filter by transaction ID"),
                        ("tx_index" = Option<i32>, Query, description = "Filter by transaction index"),
                        ("output_index" = Option<i32>, Query, description = "Filter by output index"),
                        ("block_height" = Option<BlockHeight>, Query, description = "Filter by block height"),
                        ("to_address" = Option<Address>, Query, description = "Filter by recipient address (for coin, change, and variable outputs)"),
                        ("asset_id" = Option<AssetId>, Query, description = "Filter by asset ID (for coin, change, and variable outputs)"),
                        ("contract_id" = Option<ContractId>, Query, description = "Filter by contract ID (for contract and contract_created outputs)"),
                        ("timestamp" = Option<BlockTimestamp>, Query, description = "Filter by exact block timestamp"),
                        ("time_range" = Option<TimeRange>, Query, description = "Filter by time range"),
                        ("from_block" = Option<BlockHeight>, Query, description = "Filter from specific block height"),
                        ("after" = Option<Cursor>, Query, description = "Return outputs after this cursor"),
                        ("before" = Option<Cursor>, Query, description = "Return outputs before this cursor"),
                        ("first" = Option<i32>, Query, description = "Limit results, sorted by ascending order", minimum = 1, maximum = 100),
                        ("last" = Option<i32>, Query, description = "Limit results, sorted by descending order", minimum = 1, maximum = 100),
                        ("limit" = Option<i32>, Query, description = "Maximum number of results to return", minimum = 1, maximum = 1000),
                        ("offset" = Option<i32>, Query, description = "Number of results to skip", minimum = 0),
                        ("order_by" = Option<OrderBy>, Query, description = "Sort order (ASC or DESC)")
                    ),
                    responses(
                        (status = 200, description = concat!("Successfully retrieved ", stringify!($variant), " outputs"), body = GetDataResponse),
                        (status = 400, description = "Invalid query parameters", body = String),
                        (status = 404, description = "No outputs found", body = String),
                        (status = 500, description = "Internal server error", body = String)
                    ),
                    security(
                        ("api_key" = [])
                    )
                )]
                pub async fn [<$base_name _ $variant:snake>](
                    State(state): State<ServerState>,
                    req: Request<axum::body::Body>,
                ) -> Result<impl IntoResponse, ApiError> {
                    let mut query = ValidatedQuery::<OutputsQuery>::from_request(req, &state)
                        .await?
                        .into_inner();
                    query.set_output_type(Some(OutputType::$variant));
                    let response: GetDataResponse =
                        Output::find_many(&state.db.pool, &query).await?.try_into()?;
                    Ok(Json(response))
                }
            )*

            #[utoipa::path(
                get,
                path = $base_path,
                tag = $tag,
                params(
                    ("tx_id" = Option<TxId>, Query, description = "Filter by transaction ID"),
                    ("tx_index" = Option<i32>, Query, description = "Filter by transaction index"),
                    ("output_index" = Option<i32>, Query, description = "Filter by output index"),
                    ("output_type" = Option<OutputType>, Query, description = "Filter by output type"),
                    ("block_height" = Option<BlockHeight>, Query, description = "Filter by block height"),
                    ("to_address" = Option<Address>, Query, description = "Filter by recipient address (for coin, change, and variable outputs)"),
                    ("asset_id" = Option<AssetId>, Query, description = "Filter by asset ID (for coin, change, and variable outputs)"),
                    ("contract_id" = Option<ContractId>, Query, description = "Filter by contract ID (for contract and contract_created outputs)"),
                    ("timestamp" = Option<BlockTimestamp>, Query, description = "Filter by exact block timestamp"),
                    ("time_range" = Option<TimeRange>, Query, description = "Filter by time range"),
                    ("from_block" = Option<BlockHeight>, Query, description = "Filter from specific block height"),
                    ("after" = Option<Cursor>, Query, description = "Return outputs after this cursor"),
                    ("before" = Option<Cursor>, Query, description = "Return outputs before this cursor"),
                    ("first" = Option<i32>, Query, description = "Limit results, sorted by ascending order", minimum = 1, maximum = 100),
                    ("last" = Option<i32>, Query, description = "Limit results, sorted by descending order", minimum = 1, maximum = 100),
                    ("limit" = Option<i32>, Query, description = "Maximum number of results to return", minimum = 1, maximum = 1000),
                    ("offset" = Option<i32>, Query, description = "Number of results to skip", minimum = 0),
                    ("order_by" = Option<OrderBy>, Query, description = "Sort order (ASC or DESC)")
                ),
                responses(
                    (status = 200, description = "Successfully retrieved outputs", body = GetDataResponse),
                    (status = 400, description = "Invalid query parameters", body = String),
                    (status = 404, description = "No outputs found", body = String),
                    (status = 500, description = "Internal server error", body = String)
                ),
                security(
                    ("api_key" = [])
                )
            )]
            pub async fn $base_name(
                State(state): State<ServerState>,
                req: Request<axum::body::Body>,
            ) -> Result<impl IntoResponse, ApiError> {
                let mut query = ValidatedQuery::<OutputsQuery>::from_request(req, &state)
                    .await?
                    .into_inner();
                query.set_output_type(None);
                let response: GetDataResponse =
                    Output::find_many(&state.db.pool, &query).await?.try_into()?;
                Ok(Json(response))
            }
        }
    };
}

// Generate the output endpoints with the improved macro
generate_output_endpoints!(
    TAG_OUTPUTS,
    "/outputs",
    get_outputs,
    Coin => "/coin",
    Contract => "/contract",
    Change => "/change",
    Variable => "/variable",
    ContractCreated => "/contract_created"
);
