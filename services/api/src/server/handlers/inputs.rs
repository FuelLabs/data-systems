use axum::{
    extract::{FromRequest, State},
    http::Request,
    response::IntoResponse,
    Json,
};
use fuel_streams_core::types::*;
use fuel_streams_domains::{
    infra::repository::{Repository, ValidatedQuery},
    inputs::InputsQuery,
};
use paste::paste;

use super::open_api::TAG_INPUTS;
use crate::server::{
    errors::ApiError,
    routes::GetDataResponse,
    state::ServerState,
};

#[macro_export]
macro_rules! generate_input_endpoints {
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
                        ("inputIndex" = Option<i32>, Query, description = "Filter by input index"),
                        ("blockHeight" = Option<BlockHeight>, Query, description = "Filter by block height"),
                        ("ownerId" = Option<Address>, Query, description = "Filter by owner ID (for coin inputs)"),
                        ("assetId" = Option<AssetId>, Query, description = "Filter by asset ID (for coin inputs)"),
                        ("contractId" = Option<ContractId>, Query, description = "Filter by contract ID (for contract inputs)"),
                        ("senderAddress" = Option<Address>, Query, description = "Filter by sender address (for message inputs)"),
                        ("recipientAddress" = Option<Address>, Query, description = "Filter by recipient address (for message inputs)"),
                        ("address" = Option<Address>, Query, description = "Filter by address"),
                        ("after" = Option<i32>, Query, description = "Return inputs after this height"),
                        ("before" = Option<i32>, Query, description = "Return inputs before this height"),
                        ("first" = Option<i32>, Query, description = "Limit results, sorted by ascending block height", maximum = 100),
                        ("last" = Option<i32>, Query, description = "Limit results, sorted by descending block height", maximum = 100)
                    ),
                    responses(
                        (status = 200, description = "Successfully retrieved inputs", body = GetDataResponse),
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
                    let mut query = ValidatedQuery::<InputsQuery>::from_request(req, &state)
                        .await?
                        .into_inner();
                    query.set_input_type(Some(InputType::$variant));
                    let response: GetDataResponse =
                        Input::find_many(&state.db.pool, &query).await?.try_into()?;
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
                    ("inputIndex" = Option<i32>, Query, description = "Filter by input index"),
                    ("inputType" = Option<InputType>, Query, description = "Filter by input type"),
                    ("blockHeight" = Option<BlockHeight>, Query, description = "Filter by block height"),
                    ("ownerId" = Option<Address>, Query, description = "Filter by owner ID (for coin inputs)"),
                    ("assetId" = Option<AssetId>, Query, description = "Filter by asset ID (for coin inputs)"),
                    ("contractId" = Option<ContractId>, Query, description = "Filter by contract ID (for contract inputs)"),
                    ("senderAddress" = Option<Address>, Query, description = "Filter by sender address (for message inputs)"),
                    ("recipientAddress" = Option<Address>, Query, description = "Filter by recipient address (for message inputs)"),
                    ("address" = Option<Address>, Query, description = "Filter by address"),
                    ("after" = Option<i32>, Query, description = "Return inputs after this height"),
                    ("before" = Option<i32>, Query, description = "Return inputs before this height"),
                    ("first" = Option<i32>, Query, description = "Limit results, sorted by ascending block height", maximum = 100),
                    ("last" = Option<i32>, Query, description = "Limit results, sorted by descending block height", maximum = 100)
                ),
                responses(
                    (status = 200, description = "Successfully retrieved inputs", body = GetDataResponse),
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
                let mut query = ValidatedQuery::<InputsQuery>::from_request(req, &state)
                    .await?
                    .into_inner();
                query.set_input_type(None);
                let response: GetDataResponse =
                    Input::find_many(&state.db.pool, &query).await?.try_into()?;
                Ok(Json(response))
            }
        }
    };
}

generate_input_endpoints!(
    TAG_INPUTS,
    "/inputs",
    get_inputs,
    Message => "/message",
    Contract => "/contract",
    Coin => "/coin"
);
