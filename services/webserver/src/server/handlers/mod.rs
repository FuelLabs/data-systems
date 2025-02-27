pub mod api_key_generate;
pub mod websocket;

use actix_web::web;
use fuel_web_utils::{
    api_key::middleware::ApiKeyAuth,
    server::{
        api::with_prefixed_route,
        middlewares::password::middleware::PasswordAuth,
    },
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
                .wrap(ApiKeyAuth::new(&state.api_keys_manager, &state.db))
                .route(web::get().to({
                    move |req, body, state: web::Data<ServerState>| {
                        handlers::websocket::get_websocket(req, body, state)
                    }
                })),
        );
        cfg.service(
            web::resource(format!(
                "{}/{}",
                with_prefixed_route("key"),
                "generate"
            ))
            .wrap(PasswordAuth::new(&state.password_manager))
            .route(web::post().to({
                move |req, body, state: web::Data<ServerState>| {
                    handlers::api_key_generate::generate_api_key(
                        req, body, state,
                    )
                }
            })),
        );
    }
}
