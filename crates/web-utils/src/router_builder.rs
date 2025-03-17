use std::convert::Infallible;

use axum::{
    extract::Request,
    response::IntoResponse,
    routing::{MethodRouter, Route},
    Router,
};
use tower::{Layer, Service};

pub struct RouterBuilder<S: Send + Sync + Clone + 'static> {
    router: Router<S>,
    base_path: String,
    prefix: Option<String>,
}

impl<S: Send + Sync + Clone + 'static> RouterBuilder<S> {
    pub fn new(base_path: &str) -> Self {
        Self {
            router: Router::new(),
            base_path: Self::safe_path(base_path),
            prefix: None,
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

    pub fn with_prefix(mut self, prefix: &str) -> Self {
        self.prefix = Some(prefix.to_string());
        self
    }

    pub fn build(self) -> (String, Router<S>) {
        let path = match self.prefix {
            Some(prefix) => Self::prefixed_route(&prefix, &self.base_path),
            None => self.base_path,
        };
        (path, self.router)
    }

    fn safe_path(path: &str) -> String {
        if path.starts_with("/") {
            path.to_string()
        } else {
            format!("/{}", path)
        }
    }

    fn prefixed_route(prefix: &str, route: &str) -> String {
        if route.starts_with('/') {
            format!("{prefix}/{}", route.trim_start_matches('/'))
        } else {
            format!("{prefix}/{route}")
        }
    }
}
