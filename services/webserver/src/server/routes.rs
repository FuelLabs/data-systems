use axum::{middleware::from_fn_with_state, routing::get, Router};
use fuel_web_utils::{
    api_key::middleware::ApiKeyMiddleware,
    router_builder::RouterBuilder,
    server::server_builder::API_BASE_PATH,
};

use super::handlers;
use crate::server::state::ServerState;

pub fn create_routes(state: &ServerState) -> Router {
    let app = Router::new();
    let manager = state.api_keys_manager.clone();
    let db = state.db.clone();
    let auth_params = (manager.clone(), db.clone());

    let (ws_path, ws_router) = RouterBuilder::new("/ws")
        .root(get(handlers::websocket::get_websocket))
        .build();

    let routes =
        Router::new()
            .nest(&ws_path, ws_router)
            .layer(from_fn_with_state(
                auth_params.clone(),
                ApiKeyMiddleware::handler,
            ));

    app.nest(API_BASE_PATH, routes).with_state(state.clone())
}
