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
    Bytes32,
    ContractId,
    TxId,
};
use fuel_streams_domains::{
    queryable::{Queryable, ValidatedQuery},
    receipts::{queryable::ReceiptsQuery, ReceiptType},
};
use fuel_web_utils::api_key::ApiKey;

use crate::server::{
    errors::ApiError,
    routes::GetDataResponse,
    state::ServerState,
};

pub struct ReceiptTypeVariant(Option<ReceiptType>);

impl<S> FromRequestParts<S> for ReceiptTypeVariant
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
            p if p.ends_with("/call") => Some(ReceiptType::Call),
            p if p.ends_with("/return") => Some(ReceiptType::Return),
            p if p.ends_with("/return_data") => Some(ReceiptType::ReturnData),
            p if p.ends_with("/panic") => Some(ReceiptType::Panic),
            p if p.ends_with("/revert") => Some(ReceiptType::Revert),
            p if p.ends_with("/log") => Some(ReceiptType::Log),
            p if p.ends_with("/log_data") => Some(ReceiptType::LogData),
            p if p.ends_with("/transfer") => Some(ReceiptType::Transfer),
            p if p.ends_with("/transfer_out") => Some(ReceiptType::TransferOut),
            p if p.ends_with("/script_result") => {
                Some(ReceiptType::ScriptResult)
            }
            p if p.ends_with("/message_out") => Some(ReceiptType::MessageOut),
            p if p.ends_with("/mint") => Some(ReceiptType::Mint),
            p if p.ends_with("/burn") => Some(ReceiptType::Burn),
            _ => None,
        };
        Ok(ReceiptTypeVariant(variant))
    }
}

#[utoipa::path(
    get,
    path = "/receipts",
    tag = "receipts",
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
pub async fn get_receipts(
    State(state): State<ServerState>,
    variant: ReceiptTypeVariant,
    req: Request<axum::body::Body>,
) -> Result<impl IntoResponse, ApiError> {
    let _api_key = ApiKey::from_req(&req)?;
    let mut query = ValidatedQuery::<ReceiptsQuery>::from_request(req, &state)
        .await?
        .into_inner();
    query.set_receipt_type(variant.0); // Use the extracted variant
    let response: GetDataResponse =
        query.execute(&state.db.pool).await?.try_into()?;
    Ok(Json(response))
}
