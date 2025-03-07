use actix_web::{web, HttpRequest, HttpResponse};
use fuel_streams_domains::{
    queryable::{Queryable, ValidatedQuery},
    receipts::{queryable::ReceiptsQuery, ReceiptType},
};
use fuel_web_utils::api_key::ApiKey;

use super::{Error, GetDataResponse};
use crate::server::state::ServerState;

pub async fn get_receipts(
    req: HttpRequest,
    req_query: ValidatedQuery<ReceiptsQuery>,
    state: web::Data<ServerState>,
    queried_receipt_type: Option<ReceiptType>,
) -> actix_web::Result<HttpResponse> {
    let _api_key = ApiKey::from_req(&req)?;
    let mut query = req_query.into_inner();
    query.set_receipt_type(queried_receipt_type);
    let response: GetDataResponse = query
        .execute(&state.db.pool)
        .await
        .map_err(Error::Sqlx)?
        .try_into()?;
    Ok(HttpResponse::Ok().json(response))
}
