use actix_web::{FromRequest, Handler, Responder};
use http::Method;
use oasgen_core::{OaOperation, OaSchema};
use super::Server;

type ResourceInner = actix_web::Resource;

pub type InnerRouteFactory = Box<dyn Fn() -> ResourceInner>;

fn build_inner_resource<F, Args>(path: &str, method: Method, handler: F) -> InnerRouteFactory
    where
        F: Handler<Args> + 'static + Copy,
        Args: FromRequest + 'static,
        F::Output: Responder + 'static,
{
    Box::new(move || {
        actix_web::Resource::new(path)
            .route(actix_web::web::route().method(method.clone()).to(handler))
    })
}

impl Server {
    // #[cfg(feature = "actix")]
    pub fn get<F, Args, Signature>(mut self, path: &str, handler: F) -> Self
        where
            F: actix_web::Handler<Args> + OaOperation<Signature> + Copy,
            Args: actix_web::FromRequest + 'static,
            F::Output: actix_web::Responder + 'static,
            <F as actix_web::Handler<Args>>::Output: OaSchema,
    {
        self.update_spec(path, Method::GET, &handler);

        self.resources.push(build_inner_resource(path, Method::GET, handler));
        self
    }

    // #[cfg(feature = "actix")]
    pub fn post<F, Args, Signature>(mut self, path: &str, handler: F) -> Self
        where
            F: actix_web::Handler<Args> + OaOperation<Signature> + Copy,
            Args: actix_web::FromRequest + 'static,
            F::Output: actix_web::Responder + 'static,
            <F as actix_web::Handler<Args>>::Output: OaSchema,
    {
        self.update_spec(path, Method::POST, &handler);

        self.resources.push(build_inner_resource(path, Method::POST, handler));

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