use axum::{
    extract::{FromRequest, FromRequestParts, State},
    http::{request::Parts, Request},
    response::IntoResponse,
    Json,
};
use fuel_streams_core::types::{
    Address,
    AssetId,
    BlockHeight,
    ContractId,
    TxId,
};
use fuel_streams_domains::{
    inputs::{queryable::InputsQuery, InputType},
    queryable::{Queryable, ValidatedQuery},
};

use super::open_api::TAG_INPUTS;
use crate::server::{
    errors::ApiError,
    routes::GetDataResponse,
    state::ServerState,
};

pub struct InputTypeVariant(Option<InputType>);

impl<S> FromRequestParts<S> for InputTypeVariant
where
    S: Send + Sync,
{
    type Rejection = ApiError;

    async fn from_request_parts(
        parts: &mut Parts,
        _state: &S,
    ) -> Result<Self, Self::Rejection> {
        let path = parts.uri.path();
        let variant = match path {
            p if p.ends_with("/message") => Some(InputType::Message),
            p if p.ends_with("/contract") => Some(InputType::Contract),
            p if p.ends_with("/coin") => Some(InputType::Coin),
            _ => None,
        };
        Ok(InputTypeVariant(variant))
    }
}

#[utoipa::path(
    get,
    path = "/inputs",
    tag = TAG_INPUTS,
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
pub async fn get_inputs(
    State(state): State<ServerState>,
    variant: InputTypeVariant,
    req: Request<axum::body::Body>,
) -> Result<impl IntoResponse, ApiError> {
    let mut query = ValidatedQuery::<InputsQuery>::from_request(req, &state)
        .await?
        .into_inner();
    if let Some(input_type) = variant.0 {
        query.set_input_type(Some(input_type));
    }
    let response: GetDataResponse =
        query.execute(&state.db.pool).await?.try_into()?;
    Ok(Json(response))
}
