use std::borrow::Borrow;
use std::sync::{Arc};
use axum::handler::Handler;
use axum::routing;
use futures::future::{Ready, ok};
use http::Method;
use indexmap::IndexMap;
use openapiv3::OpenAPI;
use axum::routing::MethodRouter;
use axum::body::Body;

use oasgen_core::{OaOperation, OaSchema};

use crate::Format;

use super::Server;

pub struct Router<S>(IndexMap<String, MethodRouter<S>>);

impl<S> Default for Router<S> {
    fn default() -> Self {
        Self(IndexMap::new())
    }
}

impl<S> Server<Router<S>, OpenAPI>
    where
        S: Clone + Send + Sync + 'static {
    pub fn axum() -> Self {
        Self::new()
    }

    fn add_route(&mut self, path: &str, route: MethodRouter<S>) {
        if path.contains('{') {
            eprintln!("WARNING: Path parameters are specified with `:name` with axum, not `{{name}}`.");
        }
        match self.router.0.get_mut(path) {
            Some(method_router) => {
                let existing = std::mem::take(method_router);
                *method_router = existing.merge(route);
            }
            None => {
                self.router.0.insert(path.to_string(), route);
            }
        }
    }

    pub fn get<F, T, Signature>(mut self, path: &str, handler: F) -> Self
        where
            F: Handler<T, S, Body>,
            T: 'static,
            F: OaOperation<Signature> + Copy + Send,
    {
        self.add_handler_to_spec(path, Method::GET, &handler);
        self.add_route(path, routing::get(handler));
        self
    }

    pub fn post<F, T, Signature>(mut self, path: &str, handler: F) -> Self
        where
            F: Handler<T, S, Body>,
            T: 'static,
            F: OaOperation<Signature> + Copy + Send,
    {
        self.add_handler_to_spec(path, Method::POST, &handler);
        self.add_route(path, routing::post(handler));
        self
    }

    pub fn put<F, T, Signature>(mut self, path: &str, handler: F) -> Self
        where
            F: Handler<T, S, Body>,
            T: 'static,
            F: OaOperation<Signature> + Copy + Send,
    {
        self.add_handler_to_spec(path, Method::PUT, &handler);
        self.add_route(path, routing::put(handler));
        self
    }
}

impl<S> Server<Router<S>, Arc<OpenAPI>>
    where
        S: Clone + Send + Sync + 'static {
    pub fn into_router(self) -> axum::Router<S> {
        use axum::response::IntoResponse;

        let mut router = axum::Router::new();
        for (path, inner) in self.router.0 {
            router = router.route(&path, inner);
        }

        if let Some(json_route) = &self.json_route {
            let spec = self.openapi.as_ref();
            let bytes = serde_json::to_vec(spec).unwrap();
            router = router.route(&json_route, routing::get(|| async {
                (
                    [(
                        http::header::CONTENT_TYPE,
                        http::HeaderValue::from_str("application/json").unwrap(),
                    )],
                    bytes,
                ).into_response()
            }));
        }

        if let Some(yaml_route) = &self.yaml_route {
            let spec = self.openapi.as_ref();
            let yaml = serde_yaml::to_string(spec).unwrap();
            router = router.route(&yaml_route, routing::get(|| async {
                (
                    [(
                        http::header::CONTENT_TYPE,
                        http::HeaderValue::from_str("text/yaml").unwrap(),
                    )],
                    yaml,
                ).into_response()
            }));
        }

        #[cfg(feature = "swagger-ui")]
        if let Some(mut path) = self.swagger_ui_route {
            let swagger = self.swagger_ui.expect("Swagger UI route set but no Swagger UI is configured.");
            let handler = routing::get(|uri: http::Uri| async move {
                match swagger.handle_url(&uri) {
                    Some(response) => response.into_response(),
                    None => {
                        axum::response::Response::builder()
                            .status(http::StatusCode::NOT_FOUND)
                            .body(axum::body::boxed(Body::empty()))
                            .unwrap()
                    }
                }
            });
            if !path.ends_with('/') {
                path.push('/');
                let slash = path.clone();
                router = router.route(&path[..path.len() - 1], routing::get(|| async move {
                    axum::response::Redirect::to(&slash)
                }));
            }
            router = router
                .route(&format!("{}", &path), handler.clone())
                .route(&format!("{}*rest", &path), handler)
        }
        router
    }
}