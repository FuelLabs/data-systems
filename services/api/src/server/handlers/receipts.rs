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
    receipts::ReceiptsQuery,
};
use paste::paste;

use super::open_api::TAG_RECEIPTS;
use crate::server::{
    errors::ApiError,
    routes::GetDataResponse,
    state::ServerState,
};

#[macro_export]
macro_rules! generate_receipt_endpoints {
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
                        ("receipt_index" = Option<i32>, Query, description = "Filter by receipt index"),
                        ("block_height" = Option<BlockHeight>, Query, description = "Filter by block height"),
                        ("from" = Option<ContractId>, Query, description = "Filter by source contract ID"),
                        ("to" = Option<ContractId>, Query, description = "Filter by destination contract ID"),
                        ("contract" = Option<ContractId>, Query, description = "Filter by contract ID"),
                        ("asset" = Option<AssetId>, Query, description = "Filter by asset ID"),
                        ("sender" = Option<Address>, Query, description = "Filter by sender address"),
                        ("recipient" = Option<Address>, Query, description = "Filter by recipient address"),
                        ("sub_id" = Option<Bytes32>, Query, description = "Filter by sub ID"),
                        ("timestamp" = Option<BlockTimestamp>, Query, description = "Filter by exact block timestamp"),
                        ("time_range" = Option<TimeRange>, Query, description = "Filter by time range"),
                        ("from_block" = Option<BlockHeight>, Query, description = "Filter from specific block height"),
                        ("after" = Option<Cursor>, Query, description = "Return receipts after this cursor"),
                        ("before" = Option<Cursor>, Query, description = "Return receipts before this cursor"),
                        ("first" = Option<i32>, Query, description = "Limit results, sorted by ascending order", minimum = 1, maximum = 100),
                        ("last" = Option<i32>, Query, description = "Limit results, sorted by descending order", minimum = 1, maximum = 100),
                        ("limit" = Option<i32>, Query, description = "Maximum number of results to return", minimum = 1, maximum = 1000),
                        ("offset" = Option<i32>, Query, description = "Number of results to skip", minimum = 0),
                        ("order_by" = Option<OrderBy>, Query, description = "Sort order (ASC or DESC)")
                    ),
                    responses(
                        (status = 200, description = concat!("Successfully retrieved ", stringify!($variant), " receipts"), body = GetDataResponse),
                        (status = 400, description = "Invalid query parameters", body = String),
                        (status = 404, description = "No receipts found", body = String),
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
                    let mut query = ValidatedQuery::<ReceiptsQuery>::from_request(req, &state)
                        .await?
                        .into_inner();
                    query.set_receipt_type(Some(ReceiptType::$variant));
                    let response: GetDataResponse =
                        Receipt::find_many(&state.db.pool, &query).await?.try_into()?;
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
                    ("receipt_index" = Option<i32>, Query, description = "Filter by receipt index"),
                    ("receipt_type" = Option<ReceiptType>, Query, description = "Filter by receipt type"),
                    ("block_height" = Option<BlockHeight>, Query, description = "Filter by block height"),
                    ("from" = Option<ContractId>, Query, description = "Filter by source contract ID"),
                    ("to" = Option<ContractId>, Query, description = "Filter by destination contract ID"),
                    ("contract" = Option<ContractId>, Query, description = "Filter by contract ID"),
                    ("asset" = Option<AssetId>, Query, description = "Filter by asset ID"),
                    ("sender" = Option<Address>, Query, description = "Filter by sender address"),
                    ("recipient" = Option<Address>, Query, description = "Filter by recipient address"),
                    ("sub_id" = Option<Bytes32>, Query, description = "Filter by sub ID"),
                    ("timestamp" = Option<BlockTimestamp>, Query, description = "Filter by exact block timestamp"),
                    ("time_range" = Option<TimeRange>, Query, description = "Filter by time range"),
                    ("from_block" = Option<BlockHeight>, Query, description = "Filter from specific block height"),
                    ("after" = Option<Cursor>, Query, description = "Return receipts after this cursor"),
                    ("before" = Option<Cursor>, Query, description = "Return receipts before this cursor"),
                    ("first" = Option<i32>, Query, description = "Limit results, sorted by ascending order", minimum = 1, maximum = 100),
                    ("last" = Option<i32>, Query, description = "Limit results, sorted by descending order", minimum = 1, maximum = 100),
                    ("limit" = Option<i32>, Query, description = "Maximum number of results to return", minimum = 1, maximum = 1000),
                    ("offset" = Option<i32>, Query, description = "Number of results to skip", minimum = 0),
                    ("order_by" = Option<OrderBy>, Query, description = "Sort order (ASC or DESC)")
                ),
                responses(
                    (status = 200, description = "Successfully retrieved receipts", body = GetDataResponse),
                    (status = 400, description = "Invalid query parameters", body = String),
                    (status = 404, description = "No receipts found", body = String),
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
                let query = ValidatedQuery::<ReceiptsQuery>::from_request(req, &state)
                    .await?
                    .into_inner();
                let response: GetDataResponse =
                    Receipt::find_many(&state.db.pool, &query).await?.try_into()?;
                Ok(Json(response))
            }
        }
    };
}

// Generate the receipt endpoints with the improved macro
generate_receipt_endpoints!(
    TAG_RECEIPTS,
    "/receipts",
    get_receipts,
    Call => "/call",
    Return => "/return",
    ReturnData => "/return_data",
    Panic => "/panic",
    Revert => "/revert",
    Log => "/log",
    LogData => "/log_data",
    Transfer => "/transfer",
    TransferOut => "/transfer_out",
    ScriptResult => "/script_result",
    MessageOut => "/message_out",
    Mint => "/mint",
    Burn => "/burn"
);
