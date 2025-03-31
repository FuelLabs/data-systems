use axum::{
    extract::{FromRequest, State},
    http::Request,
    response::IntoResponse,
    Json,
};
use fuel_streams_core::types::*;
use fuel_streams_domains::{
    infra::repository::{Repository, ValidatedQuery},
    receipts::{ReceiptType, ReceiptsQuery},
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
                        ("txId" = Option<TxId>, Query, description = "Filter by transaction ID"),
                        ("txIndex" = Option<u32>, Query, description = "Filter by transaction index"),
                        ("receiptIndex" = Option<i32>, Query, description = "Filter by receipt index"),
                        ("blockHeight" = Option<BlockHeight>, Query, description = "Filter by block height"),
                        ("from" = Option<ContractId>, Query, description = "Filter by source contract ID"),
                        ("to" = Option<ContractId>, Query, description = "Filter by destination contract ID"),
                        ("contract" = Option<ContractId>, Query, description = "Filter by contract ID"),
                        ("asset" = Option<AssetId>, Query, description = "Filter by asset ID"),
                        ("sender" = Option<Address>, Query, description = "Filter by sender address"),
                        ("recipient" = Option<Address>, Query, description = "Filter by recipient address"),
                        ("subId" = Option<Bytes32>, Query, description = "Filter by sub ID"),
                        ("address" = Option<Address>, Query, description = "Filter by address"),
                        ("after" = Option<i32>, Query, description = "Return receipts after this height"),
                        ("before" = Option<i32>, Query, description = "Return receipts before this height"),
                        ("first" = Option<i32>, Query, description = "Limit results, sorted by ascending block height", maximum = 100),
                        ("last" = Option<i32>, Query, description = "Limit results, sorted by descending block height", maximum = 100)
                    ),
                    responses(
                        (status = 200, description = "Successfully retrieved receipts", body = GetDataResponse),
                        (status = 400, description = "Invalid query parameters", body = String),
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

            // Generic handler for the base path
            #[utoipa::path(
                get,
                path = $base_path,
                tag = $tag,
                params(
                    ("txId" = Option<TxId>, Query, description = "Filter by transaction ID"),
                    ("txIndex" = Option<u32>, Query, description = "Filter by transaction index"),
                    ("receiptIndex" = Option<i32>, Query, description = "Filter by receipt index"),
                    ("receiptType" = Option<ReceiptType>, Query, description = "Filter by receipt type"),
                    ("blockHeight" = Option<BlockHeight>, Query, description = "Filter by block height"),
                    ("from" = Option<ContractId>, Query, description = "Filter by source contract ID"),
                    ("to" = Option<ContractId>, Query, description = "Filter by destination contract ID"),
                    ("contract" = Option<ContractId>, Query, description = "Filter by contract ID"),
                    ("asset" = Option<AssetId>, Query, description = "Filter by asset ID"),
                    ("sender" = Option<Address>, Query, description = "Filter by sender address"),
                    ("recipient" = Option<Address>, Query, description = "Filter by recipient address"),
                    ("subId" = Option<Bytes32>, Query, description = "Filter by sub ID"),
                    ("address" = Option<Address>, Query, description = "Filter by address"),
                    ("after" = Option<i32>, Query, description = "Return receipts after this height"),
                    ("before" = Option<i32>, Query, description = "Return receipts before this height"),
                    ("first" = Option<i32>, Query, description = "Limit results, sorted by ascending block height", maximum = 100),
                    ("last" = Option<i32>, Query, description = "Limit results, sorted by descending block height", maximum = 100)
                ),
                responses(
                    (status = 200, description = "Successfully retrieved receipts", body = GetDataResponse),
                    (status = 400, description = "Invalid query parameters", body = String),
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
