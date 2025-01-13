use actix_web::web;
use fuel_web_utils::server::{
    api::with_prefixed_route,
    middlewares::auth::transform::JwtAuth,
};

use super::http::handlers::request_jwt;
use crate::server::{state::ServerState, ws::handlers::get_ws};

pub fn svc_handlers(
    state: ServerState,
) -> impl Fn(&mut web::ServiceConfig) + Send + Sync + 'static {
    move |cfg: &mut web::ServiceConfig| {
        cfg.service(
            web::resource(with_prefixed_route("jwt"))
                .route(web::post().to(request_jwt)),
        );
        cfg.service(
            web::resource(with_prefixed_route("ws"))
                .wrap(JwtAuth::new(state.jwt_secret.clone()))
                .route(web::get().to({
                    move |req, body, state: web::Data<ServerState>| {
                        get_ws(req, body, state)
                    }
                })),
        );
    }
}
