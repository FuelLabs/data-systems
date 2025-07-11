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
    messages::{Message, MessageType, MessagesQuery},
};

use super::open_api::TAG_MESSAGES;
use crate::server::{
    errors::ApiError,
    routes::GetDataResponse,
    state::ServerState,
};

#[utoipa::path(
    get,
    path = "/messages",
    tag = TAG_MESSAGES,
    params(
        ("type" = Option<MessageType>, Query, description = "Filter by message type"),
        ("sender" = Option<Address>, Query, description = "Filter by sender address"),
        ("recipient" = Option<Address>, Query, description = "Filter by recipient address"),
        ("nonce" = Option<Nonce>, Query, description = "Filter by nonce"),
        ("amount" = Option<Word>, Query, description = "Filter by amount"),
        ("data" = Option<HexData>, Query, description = "Filter by message data"),
        ("da_height" = Option<DaBlockHeight>, Query, description = "Filter by DA block height"),
        ("block_height" = Option<BlockHeight>, Query, description = "Filter by block height"),
        ("message_index" = Option<u32>, Query, description = "Filter by message index"),
        ("timestamp" = Option<BlockTimestamp>, Query, description = "Filter by exact block timestamp"),
        ("time_range" = Option<TimeRange>, Query, description = "Filter by time range"),
        ("from_block" = Option<BlockHeight>, Query, description = "Filter from specific block height"),
        ("after" = Option<Cursor>, Query, description = "Return messages after this cursor"),
        ("before" = Option<Cursor>, Query, description = "Return messages before this cursor"),
        ("first" = Option<i32>, Query, description = "Limit results, sorted by ascending order", minimum = 1, maximum = 50),
        ("last" = Option<i32>, Query, description = "Limit results, sorted by descending order", minimum = 1, maximum = 50),
        ("limit" = Option<i32>, Query, description = "Maximum number of results to return", minimum = 1, maximum = 50),
        ("offset" = Option<i32>, Query, description = "Number of results to skip", minimum = 0),
        ("order_by" = Option<OrderBy>, Query, description = "Sort order (ASC or DESC)")
    ),
    responses(
        (status = 200, description = "Successfully retrieved messages", body = GetDataResponse),
        (status = 400, description = "Invalid query parameters", body = String),
        (status = 404, description = "No messages found", body = String),
        (status = 500, description = "Internal server error", body = String)
    ),
    security(
        ("api_key" = [])
    )
)]
pub async fn get_messages(
    State(state): State<ServerState>,
    req: Request<axum::body::Body>,
) -> Result<impl IntoResponse, ApiError> {
    let query = ValidatedQuery::<MessagesQuery>::from_request(req, &state)
        .await?
        .into_inner();
    let response: GetDataResponse = Message::find_many(&state.db.pool, &query)
        .await?
        .try_into()?;
    Ok(Json(response))
}
