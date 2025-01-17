pub mod websocket;

use actix_web::web;
use fuel_web_utils::server::{
    api::with_prefixed_route,
    middlewares::api_key::middleware::ApiKeyAuth,
};

use super::handlers;
use crate::server::state::ServerState;

pub fn create_services(
    state: ServerState,
) -> impl Fn(&mut web::ServiceConfig) + Send + Sync + 'static {
    move |cfg: &mut web::ServiceConfig| {
        cfg.app_data(web::Data::new(state.clone()));
        cfg.service(
            web::resource(with_prefixed_route("ws"))
                .wrap(ApiKeyAuth::new(&state.api_keys_manager))
                .route(web::get().to({
                    move |req, body, state: web::Data<ServerState>| {
                        handlers::websocket::get_websocket(req, body, state)
                    }
                })),
        );
    }
}
