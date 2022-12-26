use http::Method;
use openapiv3::{Components, OpenAPI, ReferenceOr};
use oasgen_core::{OaSchema, OaOperation};

// #[cfg(feature = "actix")]
type RouteInner = actix_web::Route;

// #[cfg(feature = "actix")]
fn into_inner<F, Args>(method: Method, handler: F) -> RouteInner where
    F: actix_web::Handler<Args>,
    Args: actix_web::FromRequest + 'static,
    F::Output: actix_web::Responder + 'static,
{
    use actix_web::web;
    web::route().method(method).to(handler)
}

struct Route {
    #[allow(unused)]
    path: String,
    #[allow(unused)]
    inner: RouteInner,
}

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
            inner: into_inner(Method::GET, handler),
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
            inner: into_inner(Method::POST, handler),
        });

        self
    }

    // #[cfg(feature = "actix")]
    fn into_service(self) -> () {
        // actix_web::web::scope("/").routes(self.resources.into_iter().map(|route| {
        //     actix_web::web::resource(&route.path).route(route.inner)
        // }))
    }
}

#[cfg(test)]
mod tests {
    // use super::*;
}