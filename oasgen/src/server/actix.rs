use actix_web::{FromRequest, Handler, HttpResponse, Resource, Responder, Scope, web};
use http::Method;

use oasgen_core::{OaOperation, OaSchema};

use crate::Format;

use super::Server;

/// ResourceFactory is a no-argument closure that returns a user-provided view handler.
///
/// Because `actix_web::Resource : !Clone`, we can't store the `Resource` directly in the `Server`
/// struct (since we need `Server: Clone`, because `Server` is cloned for every server thread by actix_web).
/// This trait essentially adds `Clone` to these closures.
pub trait ResourceFactory<'a>: Send + Fn() -> Resource {
    fn manual_clone(&self) -> InnerResourceFactory<'static>;
}

impl<'a, T> ResourceFactory<'a> for T
    where T: 'static + Clone + Fn() -> Resource + Send
{
    fn manual_clone(&self) -> InnerResourceFactory<'static> {
        Box::new(self.clone())
    }
}

pub type InnerResourceFactory<'a> = Box<dyn ResourceFactory<'a, Output=Resource>>;

fn build_inner_resource<F, Args>(path: String, method: Method, handler: F) -> InnerResourceFactory<'static>
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

impl Server {
    pub fn get<F, Args, Signature>(mut self, path: &str, handler: F) -> Self
        where
            F: actix_web::Handler<Args> + OaOperation<Signature> + Copy + Send,
            Args: actix_web::FromRequest + 'static,
            F::Output: actix_web::Responder + 'static,
            <F as actix_web::Handler<Args>>::Output: OaSchema,
    {
        self.add_handler_to_spec(path, Method::GET, &handler);

        self.resources.push(build_inner_resource(path.to_string(), Method::GET, handler));
        self
    }

    pub fn post<F, Args, Signature>(mut self, path: &str, handler: F) -> Self
        where
            F: actix_web::Handler<Args> + OaOperation<Signature> + Copy + Send,
            Args: actix_web::FromRequest + 'static,
            F::Output: actix_web::Responder + 'static,
            <F as actix_web::Handler<Args>>::Output: OaSchema,
    {
        self.add_handler_to_spec(path, Method::POST, &handler);

        self.resources.push(build_inner_resource(path.to_string(), Method::POST, handler));

        self
    }

    pub fn into_service(self) -> Scope {
        let mut scope = web::scope(&self.prefix.unwrap_or_default());
        for resource in self.resources {
            scope = scope.service(resource());
        }
        if let Some(path) = self.json_route {
            scope = scope.service(web::resource(&path).route(web::get().to(move || async {
                HttpResponse::Ok().body("foo")
            })));
            // scope = scope.service(web::resource(path).route(web::get().to(move || {
                // let s = "foo".to_string();
                // let spec = self.openapi;
                // HttpResponse::Ok().json(spec)
                // HttpResponse::Ok().body("foo")
            // })));
            // scope = scope.service(web::resource(path).route(web::get().to(move || {
                // HttpResponse::Ok().body("foo")
                // HttpResponse::Ok().json(&self.openapi)
            // })));
        }
        if let Some(path) = self.yaml_route {
            // scope = scope.service(web::resource(path).route(web::get().to(move || {
            //     let body = serde_yaml::to_string(&self.openapi).unwrap();
            //     HttpResponse::Ok()
            //         .insert_header((http::header::CONTENT_TYPE, "text/yaml"))
            //         .body(body)
            // })));
        }
        scope
    }
}