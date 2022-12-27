use std::future::Future;
use actix_web::body::BoxBody;
use actix_web::dev::{fn_service, ServiceRequest, ServiceResponse};
use actix_web::{Error, FromRequest};
use actix_web::Responder;
use http::Method;
use openapiv3::{Components, OpenAPI, ReferenceOr};

use oasgen_core::{OaOperation, OaSchema};

// #[cfg(feature = "actix")]
type RouteInner = actix_web::Route;

// pub struct CloneableRoute {
//     method: Method,
//     handler: Box<dyn actix_web::Handler<dyn FromRequest>>
// }
use actix_service::boxed::{BoxFuture, BoxService, BoxServiceFactory};
use actix_service::ServiceFactory;

type BoxedHttpServiceFactory = BoxServiceFactory<(), ServiceRequest, ServiceResponse<BoxBody>, Error, ()>;

type MyServiceFactory<Conf, Req, Res, Err, InitErr> = dyn ServiceFactory<
    Req,
    Config=Conf,
    Response=Res,
    Error=Err,
    InitError=InitErr,
    Service=BoxService<Req, Res, Err>,
    Future=BoxFuture<Result<BoxService<Req, Res, Err>, InitErr>>,
>;


fn make_service_factory<F, Args>(handler: F) -> Box<MyServiceFactory<(), ServiceRequest, ServiceResponse<BoxBody>, Error, ()>>
    where
        F: actix_web::Handler<Args> + 'static,
        Args: FromRequest + 'static,
        F::Output: actix_web::Responder + 'static,
{
    let z = fn_service(move |req: ServiceRequest| {
        let handler = handler.clone();

        async move {
            let (req, mut payload) = req.into_parts();

            let res = match Args::from_request(&req, &mut payload).await {
                Err(err) => actix_web::HttpResponse::from_error(err),
                Ok(data) => handler
                    .call(data)
                    .await
                    .respond_to(&req)
                    .map_into_boxed_body(),
            };
            Ok::<ServiceResponse, actix_web::Error>(actix_web::dev::ServiceResponse::new(req, res))
        }
    });
    Box::new(z)
}

// #[cfg(feature = "actix")]
fn into_inner<F, Args>(method: Method, handler: F) -> RouteInner where
    F: actix_web::Handler<Args>,
    Args: actix_web::FromRequest + 'static,
    F::Output: actix_web::Responder + 'static,
{
    use actix_web::web;
    web::route().method(method).to(handler)
}

#[derive(Clone)]
struct Route {
    #[allow(unused)]
    path: String,
    // #[allow(unused)]
    // inner: Box<dyn NewTrait<Output=RouteInner>>,
}

#[derive(Clone)]
pub struct Server {
    pub openapi: OpenAPI,
    // pub routes: Vec<(String, Method>
    resources: Vec<Route>,
}

impl Server {
    pub fn new() -> Self {
        Self {
            openapi: OpenAPI {
                components: Some(Components::default()),
                ..OpenAPI::default()
            },
            resources: vec![],
        }
    }

    fn update_spec<F, Signature>(&mut self, path: &str, _method: Method, _handler: &F)
        where
            F: OaOperation<Signature>,
    {
        let item = self.openapi.paths.paths.entry(path.to_string()).or_default();
        let item = item.as_mut().expect("Currently don't support references for PathItem");
        item.get = Some(F::operation());

        for reference in F::references() {
            if !self.openapi.schemas().contains_key(reference) {
                let schema = F::referenced_schema(reference);
                self.openapi.schemas_mut().insert(reference.to_string(), ReferenceOr::Item(schema));
            }
        }
    }

    // #[cfg(feature = "actix")]
    pub fn get<F, Args, Signature>(mut self, path: &str, handler: F) -> Self
        where
            F: actix_web::Handler<Args> + OaOperation<Signature>,
            Args: actix_web::FromRequest + 'static,
            F::Output: actix_web::Responder + 'static,
            <F as actix_web::Handler<Args>>::Output: OaSchema,
    {
        self.update_spec(path, Method::GET, &handler);

        self.resources.push(Route {
            path: path.to_string(),
            // inner: Box::new(move || into_inner(Method::GET, handler)),
        });

        self
    }

    // #[cfg(feature = "actix")]
    pub fn post<F, Args, Signature>(mut self, path: &str, handler: F) -> Self
        where
            F: actix_web::Handler<Args> + OaOperation<Signature>,
            Args: actix_web::FromRequest + 'static,
            F::Output: actix_web::Responder + 'static,
            <F as actix_web::Handler<Args>>::Output: OaSchema,
    {
        self.update_spec(path, Method::POST, &handler);

        self.resources.push(Route {
            path: path.to_string(),
            // inner: Box::new(move || into_inner(Method::POST, handler.clone())),
        });

        self
    }

    // #[cfg(feature = "actix")]
    pub fn create_service(&self) -> actix_web::Scope {
        let mut s = actix_web::Scope::new("/");
        for route in &self.resources {
            // s = s.route(&route.path, (route.inner)());
        }
        s
        // actix_web::web::scope("/").routes(self.resources.into_iter().map(|route| {
        //     actix_web::web::resource(&route.path).route(route.inner)
        // }))
    }
}

#[cfg(test)]
mod tests {
    // use super::*;
}