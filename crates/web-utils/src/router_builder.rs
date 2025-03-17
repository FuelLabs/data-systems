use std::convert::Infallible;

use axum::{
    extract::Request,
    response::IntoResponse,
    routing::{MethodRouter, Route},
    Router,
};
use tower::{Layer, Service};

use super::server::server_builder::with_prefixed_route;

pub struct RouterBuilder<S: Send + Sync + Clone + 'static> {
    router: Router<S>,
    base_path: String,
}

impl<S: Send + Sync + Clone + 'static> RouterBuilder<S> {
    pub fn new(base_path: &str) -> Self {
        Self {
            router: Router::new(),
            base_path: with_prefixed_route(base_path),
        }
    }

    pub fn path(&self) -> String {
        self.base_path.to_string()
    }

    pub fn root(mut self, handler: MethodRouter<S>) -> Self {
        self.router = self.router.route("/", handler);
        self
    }

    pub fn related(mut self, path: &str, handler: MethodRouter<S>) -> Self {
        self.router = self.router.route(path, handler);
        self
    }

    pub fn typed_routes(
        mut self,
        routes: &[&str],
        handler: MethodRouter<S>,
    ) -> Self {
        for route in routes {
            let route = format!("/{}", route);
            self.router = self.router.route(&route, handler.clone())
        }
        self
    }

    pub fn with_layer<L>(mut self, layer: L) -> Self
    where
        L: Layer<Route> + Clone + Send + Sync + 'static,
        L::Service: Service<Request> + Clone + Send + Sync + 'static,
        <L::Service as Service<Request>>::Response: IntoResponse + 'static,
        <L::Service as Service<Request>>::Error: Into<Infallible> + 'static,
        <L::Service as Service<Request>>::Future: Send + 'static,
    {
        self.router = self.router.layer(layer);
        self
    }

    pub fn build(self) -> (String, Router<S>) {
        let path = self.path();
        (path, self.router)
    }
}
