#[cfg(feature = "actix")]
mod actix;
#[cfg(not(any(feature = "actix")))]
mod none;

use std::future::Future;
use std::marker::PhantomData;
use std::sync::Arc;
use http::Method;
use openapiv3::{Components, OpenAPI, ReferenceOr};

use oasgen_core::{OaOperation, OaSchema};

#[cfg(feature = "actix")]
use self::actix::InnerResourceFactory;

#[cfg(not(any(feature = "actix")))]
use self::none::InnerResourceFactory;

// This takes a generic, Mutability, the idea being, once we decide the OpenAPI spec is finalized, we
// no longer need the ability to modify it.
pub struct Server<Mutability = OpenAPI> {
    resources: Vec<InnerResourceFactory<'static>>,

    // This is behind an arc because the handlers need to be able to clone it, and they're async,
    // extending their lifetime.
    pub openapi: Mutability,
    /// Configuration to mount the API routes (including the OpenAPI spec routes) under a path prefix.
    pub prefix: Option<String>,
    /// Configuration to serve the spec as JSON method=GET, path=`json_route`
    pub json_route: Option<String>,
    /// Configuration to serve the spec as YAML method=GET, path=`json_route`
    pub yaml_route: Option<String>,
}

impl Clone for Server<Arc<OpenAPI>> {
    fn clone(&self) -> Server<Arc<OpenAPI>> {
        Server {
            resources: self.resources.iter()
                .map(|f| f.manual_clone())
                .collect::<Vec<_>>(),

            openapi: self.openapi.clone(),
            json_route: self.json_route.clone(),
            yaml_route: self.yaml_route.clone(),
            prefix: self.prefix.clone(),
        }
    }
}

impl Server {
    pub fn new() -> Self {
        Self {
            openapi: OpenAPI {
                components: Some(Components::default()),
                ..OpenAPI::default()
            },
            resources: vec![],
            json_route: None,
            yaml_route: None,
            prefix: None,
        }
    }

    fn add_handler_to_spec<F, Signature>(&mut self, path: &str, _method: Method, _handler: &F)
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

    /// Configure the server to add a route that serves the spec as JSON
    /// ```ignore
    /// Server::new()
    ///     .route_json_spec("/openapi.json")
    ///
    /// $ curl localhost:5000/openapi.json  # 200 OK
    /// ```
    ///
    /// If you need to customize this route, manually create one. Check the README section
    /// [Route to return the spec] for tips on manually creating the route.
    pub fn route_json_spec(mut self, path: &str) -> Self {
        self.json_route = Some(path.to_string());
        self
    }

    /// Configure the server to add a route that serves the spec as JSON
    /// ```ignore
    /// Server::new()
    ///     .route_yaml_spec("/openapi.yaml")
    ///
    /// $ curl localhost:5000/openapi.json  # 200 OK
    /// ```
    ///
    /// If you need to customize this route, manually create one. Check the README section
    /// [Route to return the spec] for tips on manually creating the route.
    pub fn route_yaml_spec(mut self, path: &str) -> Self {
        self.yaml_route = Some(path.to_string());
        self
    }

    pub fn prefix(mut self, prefix: &str) -> Self {
        self.prefix = Some(prefix.to_string());
        self
    }

    /// Moves the OpenAPI spec into an Arc, so that it can be cloned and shared with view handlers.
    /// Enables `Server::clone()` (which doesn't make sense without freezing, because then modifying
    /// the schama on one Server wouldn't affect the other).
    /// You don't need to do this if you're not using a web server.
    pub fn freeze(self) -> Server<Arc<OpenAPI>> {
        Server {
            resources: self.resources,
            openapi: Arc::new(self.openapi),
            json_route: self.json_route,
            yaml_route: self.yaml_route,
            prefix: self.prefix,
        }
    }
}