#[cfg(feature = "swagger-ui")]
use actix_web::HttpRequest;
use actix_web::{web, Error, FromRequest, Handler, HttpResponse, Resource, Responder, Scope};
use futures::future::{ok, Ready};
use http::Method;
use openapiv3::OpenAPI;
use std::sync::Arc;

use crate::Format;

use super::Server;

use oasgen_core::OaSchema;

#[derive(Default)]
pub struct ActixRouter(Vec<InnerResourceFactory<'static>>);

impl Clone for ActixRouter {
    fn clone(&self) -> Self {
        ActixRouter(self.0.iter().map(|f| f.manual_clone()).collect::<Vec<_>>())
    }
}

/// ResourceFactory is a no-argument closure that returns a user-provided view handler.
///
/// Because `actix_web::Resource : !Clone`, we can't store the `Resource` directly in the `Server`
/// struct (since we need `Server: Clone`, because `Server` is cloned for every server thread by actix_web).
/// This trait essentially adds `Clone` to these closures.
pub trait ResourceFactory<'a>: Send + Fn() -> Resource {
    fn manual_clone(&self) -> InnerResourceFactory<'static>;
}

impl<'a, T> ResourceFactory<'a> for T
where
    T: 'static + Clone + Fn() -> Resource + Send,
{
    fn manual_clone(&self) -> InnerResourceFactory<'static> {
        Box::new(self.clone())
    }
}

pub type InnerResourceFactory<'a> = Box<dyn ResourceFactory<'a, Output = Resource>>;

fn build_inner_resource<F, Args>(
    path: String,
    method: Method,
    handler: F,
) -> InnerResourceFactory<'static>
where
    F: Handler<Args> + 'static + Copy + Send,
    Args: FromRequest + 'static,
    F::Output: Responder + 'static,
{
    Box::new(move || {
        actix_web::Resource::new(path.clone())
            .route(actix_web::web::route().method(method.clone()).to(handler))
    })
}

impl Server<ActixRouter> {
    pub fn actix() -> Self {
        Self::new()
    }

    pub fn get<F, Args>(mut self, path: &str, handler: F) -> Self
    where
        F: actix_web::Handler<Args>,
        Args: actix_web::FromRequest + 'static,
        F::Output: actix_web::Responder + 'static,
        <F as actix_web::Handler<Args>>::Output: OaSchema,
        F: Copy + Send,
    {
        self.add_handler_to_spec(path, Method::GET, &handler);
        self.router
            .0
            .push(build_inner_resource(path.to_string(), Method::GET, handler));
        self
    }

    pub fn post<F, Args>(mut self, path: &str, handler: F) -> Self
    where
        F: actix_web::Handler<Args> + Copy + Send,
        Args: actix_web::FromRequest + 'static,
        F::Output: actix_web::Responder + 'static,
        <F as actix_web::Handler<Args>>::Output: OaSchema,
    {
        self.add_handler_to_spec(path, Method::POST, &handler);
        self.router.0.push(build_inner_resource(
            path.to_string(),
            Method::POST,
            handler,
        ));
        self
    }
}

impl Server<ActixRouter, Arc<OpenAPI>> {
    pub fn into_service(self) -> Scope {
        let mut scope = web::scope(&self.prefix.unwrap_or_default());
        for resource in self.router.0 {
            scope = scope.service(resource());
        }
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
        scope
    }
}

#[derive(Clone)]
struct OaSpecJsonHandler(Arc<openapiv3::OpenAPI>);

impl actix_web::dev::Handler<()> for OaSpecJsonHandler {
    type Output = Result<HttpResponse, Error>;
    type Future = Ready<Self::Output>;

    fn call(&self, _: ()) -> Self::Future {
        ok(HttpResponse::Ok().json(&*self.0))
    }
}

#[derive(Clone)]
struct OaSpecYamlHandler(Arc<openapiv3::OpenAPI>);

impl actix_web::dev::Handler<()> for OaSpecYamlHandler {
    type Output = Result<HttpResponse, Error>;
    type Future = Ready<Self::Output>;

    fn call(&self, _: ()) -> Self::Future {
        let yaml = serde_yaml::to_string(&*self.0).unwrap();
        ok(HttpResponse::Ok()
            .insert_header(("Content-Type", "text/yaml"))
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
        let mut builder = HttpResponse::build(response.status());

        response.headers().iter().for_each(|(k, v)| {
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
