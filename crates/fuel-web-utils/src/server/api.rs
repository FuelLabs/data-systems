use std::{
    net::{Ipv4Addr, SocketAddrV4},
    sync::Arc,
};

use actix_cors::Cors;
use actix_server::{Server, ServerHandle};
use actix_web::{
    http::{self, Method},
    middleware::{Compress, Logger as ActixLogger},
    web::{self, ServiceConfig},
    App,
    HttpServer,
};
use tokio::task::JoinHandle;
use tracing_actix_web::TracingLogger;

use super::{
    http::handlers::{get_health, get_metrics},
    state::StateProvider,
};
use crate::MAX_WORKERS;

const API_VERSION: &str = "v1";

pub fn with_prefixed_route(route: &str) -> String {
    format!("/api/{}/{}", API_VERSION, route)
}

type ConfigureRoutes =
    Option<Arc<dyn Fn(&mut ServiceConfig) + Send + Sync + 'static>>;

pub struct ApiServerBuilder<T: StateProvider + 'static> {
    port: u16,
    state: Arc<T>,
    configure_routes: ConfigureRoutes,
}

impl<T: StateProvider> ApiServerBuilder<T> {
    pub fn new(port: u16, state: T) -> Self {
        Self {
            port,
            state: Arc::new(state),
            configure_routes: None,
        }
    }

    /// Add dynamic routes to the server
    pub fn with_dynamic_routes<F>(mut self, configure: F) -> Self
    where
        F: Fn(&mut ServiceConfig) + Send + Sync + 'static,
    {
        self.configure_routes = Some(Arc::new(configure));
        self
    }

    /// Build and run the server
    pub fn build(self) -> anyhow::Result<actix_web::dev::Server> {
        let server_addr = std::net::SocketAddr::V4(SocketAddrV4::new(
            Ipv4Addr::UNSPECIFIED,
            self.port,
        ));
        let state = self.state.clone();
        let configure_routes = self.configure_routes.clone();

        let server = HttpServer::new(move || {
            let state = state.clone();

            // Create CORS middleware
            let cors = Cors::default()
                .allow_any_origin()
                .allowed_methods(vec![
                    Method::GET,
                    Method::POST,
                    Method::PUT,
                    Method::OPTIONS,
                    Method::DELETE,
                    Method::PATCH,
                    Method::TRACE,
                ])
                .allowed_headers(vec![
                    http::header::AUTHORIZATION,
                    http::header::ACCEPT,
                ])
                .allowed_header(http::header::CONTENT_TYPE)
                .max_age(3600);

            App::new()
                .app_data(web::Data::new(state))
                .wrap(ActixLogger::default())
                .wrap(TracingLogger::default())
                .wrap(Compress::default())
                .wrap(cors)
                // Mandatory routes
                .service(
                    web::resource(with_prefixed_route("health")).route(web::get().to(get_health::<T>)),
                )
                .service(
                    web::resource(with_prefixed_route("metrics")).route(web::get().to(get_metrics::<T>)),
                )
                // Optional custom routes
                .configure(|cfg| {
                    if let Some(configure_routes) = configure_routes.as_ref() {
                        configure_routes(cfg);
                    }
                })
        })
        .bind(server_addr)?
        .workers(*MAX_WORKERS) // or any configurable value
        .shutdown_timeout(20)
        .run();

        Ok(server)
    }
}

pub async fn spawn_web_server(server: Server) -> JoinHandle<()> {
    tokio::spawn(async move {
        tracing::info!("Starting actix server ...");
        if let Err(err) = server.await {
            tracing::error!("Actix Web server error: {:?}", err);
        }
    })
}

pub async fn build_and_spawn_web_server<
    T: StateProvider + Send + Sync + 'static,
>(
    port: u16,
    state: T,
) -> anyhow::Result<ServerHandle> {
    let server = ApiServerBuilder::new(port, state).build()?;
    let server_handle = server.handle();
    spawn_web_server(server).await;
    Ok(server_handle)
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use actix_web::{http, test, web, App, HttpResponse};
    use pretty_assertions::assert_eq;

    use crate::server::{
        api::with_prefixed_route,
        state::{DefaultHealthResponse, DefaultServerState, StateProvider},
    };

    #[actix_web::test]
    async fn test_default_health_route() {
        let state = DefaultServerState::new();
        let test_route = with_prefixed_route("health");

        let app = test::init_service(
            App::new().app_data(web::Data::new(state.clone())).route(
                &test_route,
                web::get().to(
                    |state: web::Data<DefaultServerState>| async move {
                        if !state.is_healthy().await {
                            return HttpResponse::ServiceUnavailable()
                                .body("Service Unavailable");
                        }
                        HttpResponse::Ok().json(state.get_health().await)
                    },
                ),
            ),
        )
        .await;

        let uptime = Duration::from_secs(2);
        tokio::time::sleep(uptime).await;

        let req = test::TestRequest::get().uri(&test_route).to_request();
        let resp = test::call_service(&app, req).await;

        assert_eq!(resp.status(), http::StatusCode::OK);

        let result: DefaultHealthResponse = test::read_body_json(resp).await;
        assert!(result.uptime >= uptime.as_secs());
    }

    #[actix_web::test]
    async fn test_default_metrics_route() {
        let state = DefaultServerState::new();
        let test_route = with_prefixed_route("metrics");

        let app = test::init_service(
            App::new().app_data(web::Data::new(state.clone())).route(
                &test_route,
                web::get().to(
                    |state: web::Data<DefaultServerState>| async move {
                        HttpResponse::Ok().json(state.get_metrics().await)
                    },
                ),
            ),
        )
        .await;

        let uptime = Duration::from_secs(2);
        tokio::time::sleep(uptime).await;

        let req = test::TestRequest::get().uri(&test_route).to_request();
        let resp = test::call_service(&app, req).await;

        assert_eq!(resp.status(), http::StatusCode::OK);

        let result: String = test::read_body_json(resp).await;
        assert!(result.contains("uptime"));
    }
}
