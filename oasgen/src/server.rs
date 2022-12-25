use http::Method;
use openapiv3::{Components, OpenAPI, Operation, PathItem, ReferenceOr};
use oasgen_core::{OaOperation, OaSchema};

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
    path: String,
    inner: RouteInner,
}

pub struct Server {
    pub openapi: OpenAPI,
    // pub routes: Vec<(String, Method>
    resources: Vec<Route>
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

    pub fn get<F, Args>(mut self, path: &str, handler: F) -> Self
    where
        F: actix_web::Handler<Args> + OaOperation<Args, F::Future, F::Output>,
        Args: actix_web::FromRequest + 'static,
        F::Output: actix_web::Responder + 'static,
        <F as actix_web::Handler<Args>>::Output: OaSchema,
    {
        let item = self.openapi.paths.paths.entry(path.to_string()).or_default();
        let item = item.as_mut().expect("Currently don't support references for PathItem");
        item.get = Some(F::operation());

        if let Some(schema) = F::Output::schema() {
            self.openapi.schemas_mut().insert(<F::Output as OaSchema>::name().unwrap().to_string(), ReferenceOr::Item(schema));
        }

        self.resources.push(Route {
            path: path.to_string(),
            inner: into_inner(Method::GET, handler),
        });

        self
    }
}

#[cfg(test)]
mod tests {
    // use super::*;
}