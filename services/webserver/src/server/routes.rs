use axum::{
    middleware::from_fn_with_state,
    routing::{get, post},
    Router,
};
use fuel_web_utils::{
    api_key::middleware::ApiKeyMiddleware,
    router_builder::RouterBuilder,
    server::{
        middlewares::password::middleware::PasswordAuthLayer,
        server_builder::with_prefixed_route,
    },
};

use super::handlers;
use crate::server::state::ServerState;

pub fn create_routes(state: &ServerState) -> Router {
    let app = Router::new();

    let manager = state.api_keys_manager.clone();
    let db = state.db.clone();

    let (ws_path, ws_router) = RouterBuilder::new("ws")
        .root(get(handlers::websocket::get_websocket))
        .with_layer(from_fn_with_state(
            (manager, db),
            ApiKeyMiddleware::handler,
        ))
        .build();

    let (key_path, key_router) = RouterBuilder::new("key")
        .related(
            "generate",
            post(handlers::api_key_generate::generate_api_key),
        )
        .with_layer(PasswordAuthLayer::new(state.password_manager.clone()))
        .build();

    app.nest(&with_prefixed_route(&ws_path), ws_router)
        .nest(&with_prefixed_route(&key_path), key_router)
        .with_state(state.to_owned())
}
