use axum::{
    extract::{FromRequest, State},
    http::Request,
    response::IntoResponse,
    Json,
};
use fuel_streams_core::types::*;
use fuel_streams_domains::{
    infra::repository::{Repository, ValidatedQuery},
    outputs::{OutputType, OutputsQuery},
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
                        ("txId" = Option<TxId>, Query, description = "Filter by transaction ID"),
                        ("txIndex" = Option<u32>, Query, description = "Filter by transaction index"),
                        ("outputIndex" = Option<i32>, Query, description = "Filter by output index"),
                        ("blockHeight" = Option<BlockHeight>, Query, description = "Filter by block height"),
                        ("toAddress" = Option<Address>, Query, description = "Filter by recipient address (for coin, change, and variable outputs)"),
                        ("assetId" = Option<AssetId>, Query, description = "Filter by asset ID (for coin, change, and variable outputs)"),
                        ("contractId" = Option<ContractId>, Query, description = "Filter by contract ID (for contract and contract_created outputs)"),
                        ("address" = Option<Address>, Query, description = "Filter by address"),
                        ("after" = Option<i32>, Query, description = "Return outputs after this height"),
                        ("before" = Option<i32>, Query, description = "Return outputs before this height"),
                        ("first" = Option<i32>, Query, description = "Limit results, sorted by ascending block height", maximum = 100),
                        ("last" = Option<i32>, Query, description = "Limit results, sorted by descending block height", maximum = 100)
                    ),
                    responses(
                        (status = 200, description = "Successfully retrieved outputs", body = GetDataResponse),
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
                    let mut query = ValidatedQuery::<OutputsQuery>::from_request(req, &state)
                        .await?
                        .into_inner();
                    query.set_output_type(Some(OutputType::$variant));
                    let response: GetDataResponse =
                        Output::find_many(&state.db.pool, &query).await?.try_into()?;
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
                    ("outputIndex" = Option<i32>, Query, description = "Filter by output index"),
                    ("outputType" = Option<OutputType>, Query, description = "Filter by output type"),
                    ("blockHeight" = Option<BlockHeight>, Query, description = "Filter by block height"),
                    ("toAddress" = Option<Address>, Query, description = "Filter by recipient address (for coin, change, and variable outputs)"),
                    ("assetId" = Option<AssetId>, Query, description = "Filter by asset ID (for coin, change, and variable outputs)"),
                    ("contractId" = Option<ContractId>, Query, description = "Filter by contract ID (for contract and contract_created outputs)"),
                    ("address" = Option<Address>, Query, description = "Filter by address"),
                    ("after" = Option<i32>, Query, description = "Return outputs after this height"),
                    ("before" = Option<i32>, Query, description = "Return outputs before this height"),
                    ("first" = Option<i32>, Query, description = "Limit results, sorted by ascending block height", maximum = 100),
                    ("last" = Option<i32>, Query, description = "Limit results, sorted by descending block height", maximum = 100)
                ),
                responses(
                    (status = 200, description = "Successfully retrieved outputs", body = GetDataResponse),
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
