pub mod http;
pub mod websocket;

use actix_web::web;
use fuel_web_utils::server::{
    api::with_prefixed_route,
    middlewares::auth::transform::JwtAuth,
};

use super::handlers;
use crate::server::state::ServerState;

pub fn create_services(
    state: ServerState,
) -> impl Fn(&mut web::ServiceConfig) + Send + Sync + 'static {
    move |cfg: &mut web::ServiceConfig| {
        cfg.service(
            web::resource(with_prefixed_route("jwt"))
                .route(web::post().to(handlers::http::request_jwt)),
        );
        cfg.service(
            web::resource(with_prefixed_route("ws"))
                .wrap(JwtAuth::new(state.jwt_secret.clone()))
                .route(web::get().to({
                    move |req, body, state: web::Data<ServerState>| {
                        handlers::websocket::get_ws(req, body, state)
                    }
                })),
        );
    }
}
