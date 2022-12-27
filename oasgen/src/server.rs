#[cfg(feature = "actix")]
mod actix;
#[cfg(not(any(feature = "actix")))]
mod none;

use std::env::var;
use std::future::Future;
use std::marker::PhantomData;
use std::path::Path;
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

    /// Add a handler to the OpenAPI spec (which is different than mounting it to a server).
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
    /// ```
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
    /// ```
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

    /// Configure a prefix to mount the API routes (including the OpenAPI spec routes) under.
    pub fn prefix(mut self, prefix: &str) -> Self {
        self.prefix = Some(prefix.to_string());
        self
    }

    /// Convenience method
    pub fn inspect(self, closure: impl Fn(&OpenAPI)) -> Self {
        closure(&self.openapi);
        self
    }

    /// Convenience method for writing the spec to a file if the process was run with an env var set.
    /// To write your OpenAPI spec to a file during your build process:
    /// 1. Build the server executable.
    /// 2. Run the server executable with `OASGEN_WRITE_SPEC=1`.
    ///
    /// This function checks the env var, and if it's found, writes the spec, and then terminates
    /// the program (with success).
    pub fn write_and_exit_if_env_var_set<P: AsRef<Path>>(self, path: P) -> Self {
        let path = path.as_ref();
        if var("OASGEN_WRITE_SPEC").map(|s| s == "1").unwrap_or(false) {
            let spec = if path.extension().map(|e| e == "json").unwrap_or(false) {
                serde_json::to_string(&self.openapi).expect("Serializing OpenAPI spec to JSON failed.")
            } else {
                serde_yaml::to_string(&self.openapi).expect("Serializing OpenAPI spec failed.")
            };
            std::fs::write(path, spec).expect("Writing OpenAPI spec to file failed.");
            eprintln!("{}: Wrote OpenAPI spec.", path.display());
            std::process::exit(0);
        }
        self
    }
    /// Semantically, this declares we've finishing building the spec, and we're ready to serve it.
    ///
    /// Functionally, it moves the OpenAPI spec into an Arc, so that view handlers (which are async
    /// and therefore have undetermined lifespans) can hold onto it.
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