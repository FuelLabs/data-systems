use std::net::{Ipv4Addr, SocketAddrV4};

use actix_cors::Cors;
use actix_server::Server;
use actix_web::{
    http::{self, Method},
    middleware::{Compress, Logger as ActixLogger},
    web,
    App,
    HttpServer,
};
use tracing_actix_web::TracingLogger;

use super::{
    http::handlers::{get_health, get_metrics, request_jwt},
    middlewares::auth::JwtAuth,
    state::ServerState,
    ws::socket::get_ws,
};
use crate::config::Config;

const MAX_WORKERS: usize = 10;

const API_VERSION: &str = "v1";

fn with_prefixed_route(route: &str) -> String {
    format!("/api/{}/{}", API_VERSION, route)
}

pub fn create_api(
    config: &Config,
    state: ServerState,
) -> anyhow::Result<Server> {
    let server_addr = std::net::SocketAddr::V4(SocketAddrV4::new(
        Ipv4Addr::UNSPECIFIED,
        config.api.port,
    ));
    let jwt_secret = config.auth.jwt_secret.clone();
    let server = HttpServer::new(move || {
        let jwt_secret = jwt_secret.clone();
        // create cors
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
            .app_data(web::Data::new(state.clone()))
            .wrap(ActixLogger::default())
            .wrap(TracingLogger::default())
            .wrap(Compress::default())
            .wrap(cors)
            .service(
                web::resource(with_prefixed_route("health"))
                    .route(web::get().to(get_health)),
            )
            .service(
                web::resource(with_prefixed_route("metrics"))
                    .route(web::get().to(get_metrics)),
            )
            .service(
                web::resource(with_prefixed_route("jwt"))
                    .route(web::get().to(request_jwt)),
            )
            .service(
                web::resource(with_prefixed_route("ws"))
                    .wrap(JwtAuth::new(jwt_secret))
                    .route(web::get().to(get_ws)),
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
    use fuel_streams::types::FuelNetwork;

    use crate::{
        config::Config,
        server::{
            context::Context,
            state::{HealthResponse, ServerState},
        },
        telemetry::{metrics::Metrics, Telemetry},
    };

    #[actix_web::test]
    async fn test_health_check() {
        let telemetry = Telemetry::new(Some(Metrics::generate_random_prefix()))
            .await
            .unwrap();
        telemetry.start().await.unwrap();

        let mut config = Config::default();
        config.nats.network = FuelNetwork::Local;
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
