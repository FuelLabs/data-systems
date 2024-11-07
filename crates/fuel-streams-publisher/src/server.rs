use std::net::SocketAddr;

use actix_cors::Cors;
use actix_server::Server;
use actix_web::{http, web, App, HttpResponse, HttpServer};
use tracing_actix_web::TracingLogger;

use crate::server_state::ServerState;

// We are keeping this low to give room for more
// Publishing processing power. This is fine since the
// the latency tolerance when fetching /health and /metrics
// is trivial
const MAX_WORKERS: usize = 2;

pub fn create_web_server(
    state: ServerState,
    actix_server_addr: SocketAddr,
) -> anyhow::Result<Server> {
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
            .wrap(TracingLogger::default())
            .wrap(cors)
            .service(web::resource("/health").route(web::get().to(
                |state: web::Data<ServerState>| async move {
                    if !state.is_healthy() {
                        return HttpResponse::ServiceUnavailable()
                            .body("Service Unavailable");
                    }
                    HttpResponse::Ok().json(state.get_health().await)
                },
            )))
            .service(web::resource("/metrics").route(web::get().to(
                |state: web::Data<ServerState>| async move {
                    HttpResponse::Ok()
                        .body(state.publisher.telemetry.get_metrics().await)
                },
            )))
    })
    .bind(actix_server_addr)?
    .workers(MAX_WORKERS)
    .shutdown_timeout(20)
    .run();

    Ok(server)
}

#[cfg(test)]
#[cfg(feature = "test-helpers")]
mod tests {
    use std::time::Duration;

    use actix_web::{http, test, web, App, HttpResponse};
    use fuel_core::service::Config;
    use fuel_core_bin::FuelService;
    use fuel_core_services::State;
    use fuel_streams_core::prelude::NATS_URL;

    use crate::{
        server_state::{HealthResponse, ServerState},
        telemetry::Telemetry,
        FuelCore,
        Publisher,
    };

    #[actix_web::test]
    async fn test_health_check() {
        let fuel_service =
            FuelService::new_node(Config::local_node()).await.unwrap();
        assert_eq!(fuel_service.state(), State::Started);

        let telemetry = Telemetry::new().await.unwrap();

        let fuel_core = FuelCore::from(fuel_service);
        let publisher = Publisher::new(fuel_core.arc(), NATS_URL, telemetry)
            .await
            .unwrap();
        let state = ServerState::new(publisher).await;
        assert!(state.publisher.nats_client.is_connected());

        let app = test::init_service(
            App::new().app_data(web::Data::new(state.clone())).route(
                "/health",
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

        let req = test::TestRequest::get().uri("/health").to_request();
        let resp = test::call_service(&app, req).await;

        assert_eq!(resp.status(), http::StatusCode::OK);

        let result: HealthResponse = test::read_body_json(resp).await;
        assert!(result.uptime >= uptime.as_secs());
        assert!(!result.streams_info.is_empty());
    }
}
