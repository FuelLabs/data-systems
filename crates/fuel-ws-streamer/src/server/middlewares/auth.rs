// use std::collections::HashMap;
// use std::future::{ready, Ready};
// use std::pin::Pin;

// use actix_web::{
//     dev::{Service, ServiceRequest, ServiceResponse, Transform},
//     error::ErrorUnauthorized,
//     web, Error, HttpResponse,
// };
// use futures::future::{self, LocalBoxFuture, FutureExt};

// use crate::server::auth::authorize_request;
// use crate::server::state::ServerState;

// // Middleware factory
// pub struct AuthMiddleware;

// // Middleware transform definition
// impl<S, B> Transform<S, ServiceRequest> for AuthMiddleware
// where
//     S: Service<Request = ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
//     S::Future: 'static,
//     B: 'static,
// {
//     type Response = ServiceResponse<B>;
//     type Error = Error;
//     type Transform = AuthMiddlewareService<S>;
//     type InitError = ();
//     type Future = Ready<Result<Self::Transform, Self::InitError>>;

//     fn new_transform(&self, service: S) -> Self::Future {
//         ready(Ok(AuthMiddlewareService { service }))
//     }
// }

// // The actual middleware service
// pub struct AuthMiddlewareService<S> {
//     service: S,
// }

// impl<S, B> Service<S> for AuthMiddlewareService<S>
// where
//     S: Service<Request = ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
//     S::Future: 'static,
//     B: 'static,
// {
//     type Response = ServiceResponse<B>;
//     type Error = Error;
//     type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

//     fn poll_ready(&mut self, ctx: &mut std::task::Context<'_>) -> std::task::Poll<Result<(), Self::Error>> {
//         self.service.poll_ready(ctx)
//     }

//     fn call(&mut self, req: ServiceRequest) -> Self::Future {
//         // Extract the app state and the `Authorization` header
//         let jwt_secret = req
//             .app_data::<web::Data<ServerState>>()
//             .map(|data| data.context.jwt_secret.clone());

//         let headers = req.headers();
//         let auth_header = headers.get("Authorization");

//         // Perform the authorization logic
//         let is_authorized = if let Some(jwt_secret) = jwt_secret {
//             if let Some(auth_header) = auth_header {
//                 let token = auth_header.to_str().unwrap_or_default();
//                 authorize_request(jwt_secret, token).is_ok()
//             } else {
//                 false
//             }
//         } else {
//             false
//         };

//         if !is_authorized {
//             // If unauthorized, respond with 401 Unauthorized
//             let response = req.error_response(ErrorUnauthorized("Unauthorized"));
//             return future::ready(Ok(req.into_response(response))).boxed_local();
//         }

//         // If authorized, call the next service
//         let fut = self.service.call(req);
//         async move { fut.await }.boxed_local()
//     }
// }
