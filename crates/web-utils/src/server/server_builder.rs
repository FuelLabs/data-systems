use std::{
    net::{Ipv4Addr, SocketAddrV4},
    time::Duration,
};

use axum::{
    extract::{Extension, MatchedPath, Request},
    routing::get,
    Router,
};
use tower_http::{
    compression::CompressionLayer,
    cors::{Any, CorsLayer},
    decompression::RequestDecompressionLayer,
    trace::TraceLayer,
};
use tracing::info_span;

use super::{
    http::handlers::{get_health, get_metrics},
    state::StateProvider,
};

pub const API_VERSION: &str = "v1";

pub fn with_prefixed_route(route: &str) -> String {
    if route.starts_with('/') {
        format!("/api/{}/{}", API_VERSION, route.trim_start_matches('/'))
    } else {
        format!("/api/{}/{}", API_VERSION, route)
    }
}

pub struct Server {
    app: Router,
    port: u16,
}

impl Server {
    pub fn with_router(mut self, router: Router) -> Self {
        self.app = self.app.merge(router);
        self
    }

    pub async fn run(self) -> anyhow::Result<()> {
        let addr = std::net::SocketAddr::V4(SocketAddrV4::new(
            Ipv4Addr::UNSPECIFIED,
            self.port,
        ));

        let listener = tokio::net::TcpListener::bind(addr).await?;
        tracing::debug!("listening on {}", listener.local_addr().unwrap());
        axum::serve(listener, self.app)
            .with_graceful_shutdown(shutdown_signal())
            .await?;
        Ok(())
    }
}

pub struct ServerBuilder;

impl ServerBuilder {
    pub fn build<S: StateProvider + Clone + 'static>(
        state: &S,
        port: u16,
    ) -> Server {
        let app = Router::new()
            .route(&with_prefixed_route("health"), get(get_health::<S>))
            .route(&with_prefixed_route("metrics"), get(get_metrics::<S>))
            .layer(Extension(state.to_owned()))
            .layer(TraceLayer::new_for_http().make_span_with(
                |request: &Request<_>| {
                    let matched_path = request
                        .extensions()
                        .get::<MatchedPath>()
                        .map(MatchedPath::as_str);
                    info_span!(
                        "http_request",
                        method = ?request.method(),
                        matched_path,
                        some_other_field = tracing::field::Empty,
                    )
                },
            ))
            .layer(RequestDecompressionLayer::new())
            .layer(CompressionLayer::new())
            .layer(
                CorsLayer::new()
                    .allow_origin(Any)
                    .allow_methods(vec![
                        axum::http::Method::GET,
                        axum::http::Method::POST,
                        axum::http::Method::PUT,
                        axum::http::Method::OPTIONS,
                        axum::http::Method::DELETE,
                        axum::http::Method::PATCH,
                        axum::http::Method::TRACE,
                    ])
                    .allow_headers(vec![
                        axum::http::header::AUTHORIZATION,
                        axum::http::header::ACCEPT,
                        axum::http::header::CONTENT_TYPE,
                    ])
                    .max_age(Duration::from_secs(3600)),
            )
            .with_state(state.to_owned());

        Server { app, port }
    }
}

async fn shutdown_signal() {
    use tokio::signal;
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
}
