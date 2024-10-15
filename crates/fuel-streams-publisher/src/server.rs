use std::net::SocketAddr;

use actix_cors::Cors;
use actix_server::Server;
use actix_web::{http, web, App, HttpResponse, HttpServer};
use tracing_actix_web::TracingLogger;

use crate::state::SharedState;

const RUNTIME_WORKER_MULTIPLIER: usize = 2;

pub fn create_web_server(
    state: SharedState,
    actix_server_addr: SocketAddr,
) -> anyhow::Result<Server> {
    // compute worker threads
    let num_cpus = num_cpus::get();
    let worker_threads = (num_cpus * RUNTIME_WORKER_MULTIPLIER).max(16);
    tracing::info!(
        "Starting runtime: num_cpus={}, worker_threads={}",
        num_cpus,
        worker_threads
    );

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
                |state: web::Data<SharedState>| async move {
                    if !state.is_healthy() {
                        return HttpResponse::ServiceUnavailable()
                            .body("Service Unavailable");
                    }
                    HttpResponse::Ok().json(state.get_health().await)
                },
            )))
            .service(web::resource("/metrics").route(web::get().to(
                |state: web::Data<SharedState>| async move {
                    HttpResponse::Ok().body(state.metrics())
                },
            )))
    })
    .bind(actix_server_addr)?
    .workers(worker_threads)
    .shutdown_timeout(20)
    .run();

    Ok(server)
}

#[cfg(test)]
#[cfg(feature = "test-helpers")]
mod tests {
    use std::{sync::Arc, time::Duration};

    use actix_web::{http, test, web, App, HttpResponse};
    use fuel_core::service::Config;
    use fuel_core_bin::FuelService;
    use fuel_core_services::State;
    use fuel_streams_core::prelude::NATS_URL;
    use parking_lot::RwLock;

    use crate::{
        metrics::PublisherMetrics,
        state::{HealthResponse, SharedState},
        system::System,
    };

    #[actix_web::test]
    async fn test_health_check() {
        let fuel_svc = Arc::new(
            FuelService::new_node(Config::local_node()).await.unwrap(),
        );
        assert_eq!(fuel_svc.state(), State::Started);
        let state = SharedState::new(
            fuel_svc,
            NATS_URL,
            Arc::new(PublisherMetrics::random()),
            Arc::new(RwLock::new(System::new().await)),
        )
        .await
        .unwrap();
        assert!(state.nats_client.is_connected());

        let app = test::init_service(
            App::new().app_data(web::Data::new(state.clone())).route(
                "/health",
                web::get().to(|state: web::Data<SharedState>| async move {
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
