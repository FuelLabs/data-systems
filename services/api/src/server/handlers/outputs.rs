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
    outputs::{queryable::OutputsQuery, OutputType},
    queryable::{Queryable, ValidatedQuery},
};

use crate::server::{
    errors::ApiError,
    routes::GetDataResponse,
    state::ServerState,
};

pub struct OutputTypeVariant(Option<OutputType>);

impl<S> FromRequestParts<S> for OutputTypeVariant
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
            p if p.ends_with("/coin") => Some(OutputType::Coin),
            p if p.ends_with("/change") => Some(OutputType::Change),
            p if p.ends_with("/variable") => Some(OutputType::Variable),
            p if p.ends_with("/contract") => Some(OutputType::Contract),
            p if p.ends_with("/contract_created") => {
                Some(OutputType::ContractCreated)
            }
            _ => None,
        };
        Ok(OutputTypeVariant(variant))
    }
}

#[utoipa::path(
    get,
    path = "/outputs",
    tag = "outputs",
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
pub async fn get_outputs(
    State(state): State<ServerState>,
    variant: OutputTypeVariant,
    req: Request<axum::body::Body>,
) -> Result<impl IntoResponse, ApiError> {
    let mut query = ValidatedQuery::<OutputsQuery>::from_request(req, &state)
        .await?
        .into_inner();
    if let Some(output_type) = variant.0 {
        query.set_output_type(Some(output_type));
    }
    let response: GetDataResponse =
        query.execute(&state.db.pool).await?.try_into()?;
    Ok(Json(response))
}
