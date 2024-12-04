use std::net::{Ipv4Addr, SocketAddrV4};

use actix_cors::Cors;
use actix_server::Server;
use actix_web::{
    http,
    middleware::Logger as ActixLogger,
    web,
    App,
    HttpResponse,
    HttpServer,
};
use tracing_actix_web::TracingLogger;

use super::{state::ServerState, ws::ws};
use crate::config::Config;

// We are keeping this low to give room for more
// Publishing processing power. This is fine since the
// the latency tolerance when fetching /health and /metrics
// is trivial
const MAX_WORKERS: usize = 2;

const API_VERSION: &str = "v1";

pub fn create_web_server(
    config: &Config,
    state: ServerState,
) -> anyhow::Result<Server> {
    let server_addr = std::net::SocketAddr::V4(SocketAddrV4::new(
        Ipv4Addr::UNSPECIFIED,
        config.api.port,
    ));

    let server = HttpServer::new(move || {
        // create cors
        let cors = Cors::default()
            .allow_any_origin()
            .allowed_methods(vec!["GET", "POST"])
            .allowed_headers(vec![
                http::header::AUTHORIZATION,
                http::header::ACCEPT,
            ])
            .allowed_header(http::header::CONTENT_TYPE)
            .max_age(3600);

        App::new()
            .app_data(web::Data::new(state.clone()))
            .wrap(ActixLogger::default())
            .wrap(TracingLogger::default())
            .wrap(cors)
            .service(
                web::resource(format!("/api/{}/health", API_VERSION)).route(
                    web::get().to(|state: web::Data<ServerState>| async move {
                        if !state.is_healthy() {
                            return HttpResponse::ServiceUnavailable()
                                .body("Service Unavailable");
                        }
                        HttpResponse::Ok().json(state.get_health().await)
                    }),
                ),
            )
            .service(
                web::resource(format!("/api/{}/metrics", API_VERSION)).route(
                    web::get().to(|state: web::Data<ServerState>| async move {
                        HttpResponse::Ok()
                            .body(state.context.telemetry.get_metrics().await)
                    }),
                ),
            )
            .service(
                web::resource(format!("/api/{}/ws", API_VERSION))
                    .route(web::get().to(ws)),
            )
    })
    .bind(server_addr)?
    .workers(MAX_WORKERS)
    .shutdown_timeout(20)
    .run();

    Ok(server)
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use actix_web::{http, test, web, App, HttpResponse};

    use crate::{
        config::Config,
        server::{
            context::Context,
            state::{HealthResponse, ServerState},
        },
        telemetry::Telemetry,
    };

    #[actix_web::test]
    async fn test_health_check() {
        let telemetry = Telemetry::new().await.unwrap();
        telemetry.start().await.unwrap();

        let mut config = Config::default();
        config.nats.url = "nats://localhost:4222".to_string();
        let context = Context::new(&config).await.unwrap();

        let state = ServerState::new(context.clone()).await;

        let app = test::init_service(
            App::new().app_data(web::Data::new(state.clone())).route(
                "api/v1/health",
                web::get().to(|state: web::Data<ServerState>| async move {
                    if !state.is_healthy() {
                        return HttpResponse::ServiceUnavailable()
                            .body("Service Unavailable");
                    }
                    HttpResponse::Ok().json(state.get_health().await)
                }),
            ),
        )
        .await;

        let uptime = Duration::from_secs(2);
        tokio::time::sleep(uptime).await;

        let req = test::TestRequest::get().uri("/api/v1/health").to_request();
        let resp = test::call_service(&app, req).await;

        assert_eq!(resp.status(), http::StatusCode::OK);

        let result: HealthResponse = test::read_body_json(resp).await;
        assert!(result.uptime >= uptime.as_secs());
    }
}
