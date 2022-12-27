use actix_web::{FromRequest, Handler, HttpResponse, Responder, Resource};
use http::Method;
use oasgen_core::{OaOperation, OaSchema};
use crate::Format;
use super::Server;

/// ResourceFactory is a no-argument closure that returns a user-provided view handler.
///
/// Because `actix_web::Resource : !Clone`, we can't store the `Resource` directly in the `Server`
/// struct (since we need `Server: Clone`, because `Server` is cloned for every server thread by actix_web).
/// This trait essentially adds `Clone` to these closures.
pub trait ResourceFactory<'a> : Send + Fn() -> Resource {
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

    pub fn get_json_spec_at_path(mut self, path: &str) -> Self {
        let s = serde_json::to_string(&self.openapi).unwrap();
        // self.resources.push(build_inner_resource(path.to_string(), Method::GET, || {
        //     HttpResponse::Ok().json(s)
        // }));
        self
    }

    pub fn into_service(self) -> actix_web::Scope {
        let mut scope = actix_web::Scope::new("");
        for resource in self.resources {
            scope = scope.service(resource());
        }
        scope
    }
}