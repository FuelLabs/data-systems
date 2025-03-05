/// Macro to generate a complete typed resource endpoint with all type variants
#[macro_export]
macro_rules! typed_resource_endpoint {
    ($cfg:expr, $api_key_middleware:expr, $base:expr, $handler:expr, $enum_type:ident, $(($route:expr, $variant:ident)),*) => {
        let scope = web::scope(&with_prefixed_route($base))
            .wrap($api_key_middleware.clone());

        // Add base route
        let scope = scope.route(
            "",
            web::get().to({
                move |req, query, state: web::Data<ServerState>| {
                    $handler(
                        req, query, state, None,
                    )
                }
            }),
        );

        // Add typed routes
        let scope = {
            let mut s = scope;
            $(
                s = s.route(
                    $route,
                    web::get().to({
                        let handler = $handler.clone();
                        move |req, query, state: web::Data<ServerState>| {
                            handler(
                                req,
                                query,
                                state,
                                Some($enum_type::$variant),
                            )
                        }
                    }),
                );
            )*
            s
        };

        $cfg.service(scope);
    };
}

/// Macro to generate a resource endpoint with common related resources
#[macro_export]
macro_rules! related_resource_endpoint {
    ($cfg:expr, $api_key_middleware:expr, $base:expr, $id_param:expr,
     [$(($path:expr, $handler_fn:path)),*]) => {
        $cfg.service(
            web::scope(&with_prefixed_route($base))
                .wrap($api_key_middleware.clone())
                $(.route(&format!("/{{{}}}/{}", $id_param, $path), web::get().to({
                    move |req, path, query, state: web::Data<ServerState>| {
                        $handler_fn(req, path, query, state)
                    }
                })))*
        )
    };
}

/// Macro to generate a resource endpoint with a root handler and related resources
#[macro_export]
macro_rules! resource_with_related_endpoints {
    ($cfg:expr, $api_key_middleware:expr, $base:expr, $id_param:expr, $root_handler:path,
     [$(($path:expr, $handler_fn:path)),*]) => {
        $cfg.service(
            web::scope(&with_prefixed_route($base))
                .wrap($api_key_middleware.clone())
                .route("", web::get().to({
                    move |req, query, state: web::Data<ServerState>| {
                        $root_handler(req, query, state)
                    }
                }))
                $(.route(&format!("/{{{}}}/{}", $id_param, $path), web::get().to({
                    move |req, path, query, state: web::Data<ServerState>| {
                        $handler_fn(req, path, query, state)
                    }
                })))*
        )
    };
}

/// Macro to generate a simple resource endpoint with only a root route
#[macro_export]
macro_rules! simple_resource_endpoint {
    ($cfg:expr, $api_key_middleware:expr, $base:expr, $handler:path) => {
        $cfg.service(
            web::scope(&with_prefixed_route($base))
                .wrap($api_key_middleware.clone())
                .route(
                    "",
                    web::get().to({
                        move |req, query, state: web::Data<ServerState>| {
                            $handler(req, query, state)
                        }
                    }),
                ),
        )
    };
}
