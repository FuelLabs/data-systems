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
                    HttpResponse::Ok().finish()
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
