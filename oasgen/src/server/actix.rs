use crate::Format;
use actix_web::dev::{AppService, HttpServiceFactory};
use actix_web::http::Method;
#[cfg(feature = "swagger-ui")]
use actix_web::HttpRequest;
use actix_web::{guard::Guard, http::header::CONTENT_TYPE};
use actix_web::{web, Error, FromRequest, Handler, HttpResponse, Resource, Responder};
use futures::future::{ok, Ready};
use indexmap::IndexMap;
use openapiv3::{OpenAPI, PathItem, RefOr};
use std::marker::PhantomData;
use std::str::FromStr;
use std::sync::Arc;

use super::{add_handler_to_spec, HandlerSpec, Server};

use oasgen_core::{OaParameter, OaSchema};

pub struct ActixRouter(actix_web::Scope);

impl Default for ActixRouter {
    fn default() -> Self {
        Self(actix_web::Scope::new(""))
    }
}

impl Server<ActixRouter> {
    pub fn actix() -> Self {
        Self::new()
    }

    pub fn service<F>(mut self, mut factory: F) -> Self
    where
        F: HandlerSpec + HttpServiceFactory + 'static,
    {
        self.extend_handler_spec(&mut factory);
        self.router.0 = self.router.0.service(factory);
        self
    }

    pub fn get<F, Args>(mut self, path: &str, handler: F) -> Self
    where
        F: Handler<Args> + Copy + Send,
        Args: FromRequest + 'static,
        F::Output: OaParameter + Responder + 'static,
    {
        self.add_handler_to_spec(path, http::Method::GET, &handler);
        self.router.0 = self.router.0.route(path, web::get().to(handler));
        self
    }

    pub fn post<F, Args>(mut self, path: &str, handler: F) -> Self
    where
        F: Handler<Args> + Copy + Send,
        Args: FromRequest + 'static,
        F::Output: OaParameter + Responder + 'static,
    {
        self.add_handler_to_spec(path, http::Method::POST, &handler);
        self.router.0 = self.router.0.route(path, web::post().to(handler));
        self
    }

    pub fn put<F, Args>(mut self, path: &str, handler: F) -> Self
    where
        F: Handler<Args> + Copy + Send,
        Args: FromRequest + 'static,
        F::Output: OaParameter + Responder + 'static,
    {
        self.add_handler_to_spec(path, http::Method::PUT, &handler);
        self.router.0 = self.router.0.route(path, web::put().to(handler));
        self
    }

    pub fn delete<F, Args>(mut self, path: &str, handler: F) -> Self
    where
        F: Handler<Args> + Copy + Send,
        Args: FromRequest + 'static,
        F::Output: OaParameter + Responder + 'static,
    {
        self.add_handler_to_spec(path, http::Method::DELETE, &handler);
        self.router.0 = self.router.0.route(path, web::delete().to(handler));
        self
    }

    pub fn patch<F, Args>(mut self, path: &str, handler: F) -> Self
    where
        F: Handler<Args> + Copy + Send,
        Args: FromRequest + 'static,
        F::Output: OaParameter + Responder + 'static,
    {
        self.add_handler_to_spec(path, http::Method::PATCH, &handler);
        self.router.0 = self.router.0.route(path, web::patch().to(handler));
        self
    }
}

impl Server<ActixRouter, Arc<OpenAPI>> {
    pub fn into_service(self) -> actix_web::Scope {
        let mut scope = web::scope(&self.prefix.unwrap_or_default());
        if let Some(path) = self.json_route {
            scope = scope.service(
                web::resource(&path).route(web::get().to(OaSpecJsonHandler(self.openapi.clone()))),
            );
        }
        if let Some(path) = self.yaml_route {
            scope = scope.service(
                web::resource(&path).route(web::get().to(OaSpecYamlHandler(self.openapi.clone()))),
            );
        }
        #[cfg(feature = "swagger-ui")]
        if self.swagger_ui_route.is_some() && self.swagger_ui.is_some() {
            let path = self.swagger_ui_route.unwrap();
            let swagger_ui = self.swagger_ui.unwrap();
            let path = format!("{}{{tail:.*}}", path);
            scope = scope.app_data(web::Data::new(swagger_ui));
            scope = scope.service(web::resource(path).route(web::get().to(handler_swagger)));
        }
        scope.service(self.router.0)
    }
}

pub struct Scope {
    path: String,
    paths: IndexMap<String, RefOr<PathItem>>,
    inner: actix_web::Scope,
}

impl Scope {
    pub fn new(path: &str) -> Self {
        Self {
            path: path.into(),
            paths: Default::default(),
            inner: actix_web::Scope::new(path),
        }
    }

    /// Proxy for [`actix_web::Scope::guard`](https://docs.rs/actix-web/*/actix_web/struct.Scope.html#method.guard).
    ///
    /// **NOTE:** This doesn't affect spec generation.
    pub fn guard<G: Guard + 'static>(mut self, guard: G) -> Self {
        self.inner = self.inner.guard(guard);
        self
    }

    /// Wrapper for [`actix_web::Scope::route`](https://docs.rs/actix-web/*/actix_web/struct.Scope.html#method.route).
    pub fn route<F, Args>(mut self, path: &str, route: Route<F>) -> Self
    where
        F: Handler<Args> + Copy + Send,
        Args: FromRequest + 'static,
        F::Output: OaParameter + Responder + 'static,
    {
        let scoped_path = self.scoped_path(path);
        add_handler_to_spec::<F>(&mut self.paths, &scoped_path, route.method());
        self.inner = self.inner.route(path, route.inner);
        self
    }

    fn scoped_path(&self, path: &str) -> String {
        match (self.path.ends_with('/'), path.starts_with('/')) {
            (true, true) => format!("{}{}", self.path, &path[1..]),
            (true, false) | (false, true) => format!("{}{}", self.path, path),
            (false, false) => format!("{}/{}", self.path, path),
        }
    }
}

impl HandlerSpec for Scope {
    fn paths(&mut self) -> impl Iterator<Item = (String, RefOr<PathItem>)> {
        self.paths.drain(..)
    }
}

/// Wrapper for [`actix_web::web::scope`](https://docs.rs/actix-web/*/actix_web/web/fn.scope.html).
pub fn scope(path: &str) -> Scope {
    Scope::new(path)
}

impl HttpServiceFactory for Scope {
    fn register(self, config: &mut AppService) {
        self.inner.register(config);
    }
}

pub struct Route<H = ()> {
    method: Method,
    inner: actix_web::Route,
    _handler: PhantomData<H>,
}

impl Route {
    pub fn new(method: Method) -> Self {
        Self {
            method: method.clone(),
            inner: actix_web::Route::new().method(method),
            _handler: PhantomData,
        }
    }

    /// Proxy for [`actix_web::Route::guard`](https://docs.rs/actix-web/*/actix_web/struct.Route.html#method.guard).
    ///
    /// **NOTE:** This doesn't affect spec generation.
    pub fn guard<G: Guard + 'static>(mut self, guard: G) -> Self {
        self.inner = self.inner.guard(guard);
        self
    }

    /// Wrapper for [`actix_web::Route::to`](https://docs.rs/actix-web/*/actix_web/struct.Route.html#method.to)
    pub fn to<F, Args>(mut self, handler: F) -> Route<F>
    where
        F: Handler<Args> + Copy + Send,
        Args: FromRequest + 'static,
        F::Output: Responder + 'static,
    {
        Route {
            method: self.method,
            inner: self.inner.to(handler),
            _handler: PhantomData,
        }
    }
}

impl<T> Route<T> {
    fn method(&self) -> http::Method {
        match &self.method {
            &Method::GET => http::Method::GET,
            &Method::POST => http::Method::POST,
            &Method::PUT => http::Method::PUT,
            &Method::DELETE => http::Method::DELETE,
            &Method::OPTIONS => http::Method::OPTIONS,
            &Method::HEAD => http::Method::HEAD,
            &Method::PATCH => http::Method::PATCH,
            &Method::TRACE => http::Method::TRACE,
            v => panic!("Unsupported method: {v}"),
        }
    }
}

/// Wrapper for [`actix_web::web::get`](https://docs.rs/actix-web/*/actix_web/web/fn.get.html).
pub fn get() -> Route {
    Route::new(Method::GET)
}

/// Wrapper for [`actix_web::web::put`](https://docs.rs/actix-web/*/actix_web/web/fn.put.html).
pub fn put() -> Route {
    Route::new(Method::PUT)
}

/// Wrapper for [`actix_web::web::post`](https://docs.rs/actix-web/*/actix_web/web/fn.post.html).
pub fn post() -> Route {
    Route::new(Method::POST)
}

/// Wrapper for [`actix_web::web::patch`](https://docs.rs/actix-web/*/actix_web/web/fn.patch.html).
pub fn patch() -> Route {
    Route::new(Method::PATCH)
}

/// Wrapper for [`actix_web::web::delete`](https://docs.rs/actix-web/*/actix_web/web/fn.delete.html).
pub fn delete() -> Route {
    Route::new(Method::DELETE)
}

#[derive(Clone)]
struct OaSpecJsonHandler(Arc<OpenAPI>);

impl Handler<()> for OaSpecJsonHandler {
    type Output = Result<HttpResponse, Error>;
    type Future = Ready<Self::Output>;

    fn call(&self, _: ()) -> Self::Future {
        ok(HttpResponse::Ok().json(&*self.0))
    }
}

#[derive(Clone)]
struct OaSpecYamlHandler(Arc<OpenAPI>);

impl Handler<()> for OaSpecYamlHandler {
    type Output = Result<HttpResponse, Error>;
    type Future = Ready<Self::Output>;

    fn call(&self, _: ()) -> Self::Future {
        let yaml = serde_yaml::to_string(&*self.0).unwrap();
        ok(HttpResponse::Ok()
            .insert_header((CONTENT_TYPE, "text/yaml"))
            .body(yaml))
    }
}

#[cfg(feature = "swagger-ui")]
async fn handler_swagger(
    req: HttpRequest,
    data: web::Data<swagger_ui::SwaggerUi>,
) -> impl Responder {
    let url = req.path();
    if let Some(mut response) = data.handle_url(url) {
        let status = response.status().as_u16();
        // actix still using http=0.2
        let status = actix_web::http::StatusCode::from_u16(status).expect("Invalid status code");
        let mut builder = HttpResponse::build(status);

        response.headers().iter().for_each(|(k, v)| {
            // actix still using http=0.2
            let k = actix_web::http::header::HeaderName::from_str(k.as_str())
                .expect("Invalid header name");
            let v = actix_web::http::header::HeaderValue::from_bytes(v.as_bytes())
                .expect("Invalid header value");
            builder.append_header((k, v));
        });
        builder.body(response.body_mut().to_owned())
    } else {
        HttpResponse::NotFound().finish()
    }
}

#[cfg(test)]
#[cfg(feature = "swagger-ui")]
#[cfg(feature = "actix")]
mod tests {
    use super::*;
    use actix_web::{test, App};

    #[actix_web::test]
    async fn test_swagger_get_index() {
        let server = Server::actix()
            .route_yaml_spec("/docs/openapi.yaml")
            .route_json_spec("/docs/openapi.json")
            .swagger_ui("/docs/")
            .freeze();
        let app = test::init_service(App::new().service(server.into_service())).await;
        let req = test::TestRequest::get()
            .uri("/docs/index.html")
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());

        let req = test::TestRequest::get()
            .uri("/docs/openapi.yaml")
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());

        let req = test::TestRequest::get()
            .uri("/docs/openapi.json")
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
    }
}
