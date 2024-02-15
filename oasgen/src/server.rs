use std::collections::HashMap;
use std::env::var;
use std::future::Future;
use std::path::Path;
use std::sync::Arc;

use http::Method;
use once_cell::sync::Lazy;
use openapiv3::{OpenAPI, Operation, ReferenceOr, Parameter, ParameterKind};

use oasgen_core::{OaSchema};

#[cfg_attr(docsrs, doc(cfg(feature = "actix")))]
#[cfg(feature = "actix")]
mod actix;
#[cfg_attr(docsrs, doc(cfg(feature = "axum")))]
#[cfg(feature = "axum")]
mod axum;
mod none;

static OPERATION_LOOKUP: Lazy<HashMap<&'static str, &'static (dyn Fn() -> Operation + Send + Sync)>> = Lazy::new(|| {
    let mut map = HashMap::new();
    for flag in inventory::iter::<oasgen_core::OperationRegister> {
        let z: &'static (dyn Fn() -> Operation + Send + Sync) = flag.constructor;
        map.insert(flag.name, z);
    }
    map
});

pub struct Server<Router, Mutability = OpenAPI> {
    router: Router,

    /// This is behind an arc because the handlers need to be able to clone it, and they're async,
    /// extending their lifetime.
    pub openapi: Mutability,
    /// Configuration to mount the API routes (including the OpenAPI spec routes) under a path prefix.
    pub prefix: Option<String>,
    /// Configuration to serve the spec as JSON
    pub json_route: Option<String>,
    /// Configuration to serve the spec as YAML
    pub yaml_route: Option<String>,

    #[cfg(feature = "swagger-ui")]
    #[cfg_attr(docsrs, doc(cfg(feature = "swagger-ui")))]
    /// Configuration for route to serve Swagger UI
    pub swagger_ui_route: Option<String>,
    #[cfg_attr(docsrs, doc(cfg(feature = "swagger-ui")))]
    #[cfg(feature = "swagger-ui")]
    /// Configuration for Swagger UI itself
    pub swagger_ui: Option<swagger_ui::SwaggerUi>,
}

impl<Router: Clone> Clone for Server<Router, Arc<OpenAPI>> {
    fn clone(&self) -> Self {
        Server {
            router: self.router.clone(),
            openapi: self.openapi.clone(),
            json_route: self.json_route.clone(),
            yaml_route: self.yaml_route.clone(),
            prefix: self.prefix.clone(),
            #[cfg(feature = "swagger-ui")]
            swagger_ui_route: self.swagger_ui_route.clone(),
            #[cfg(feature = "swagger-ui")]
            swagger_ui: self.swagger_ui.clone(),
        }
    }
}

impl<Router: Default> Server<Router, OpenAPI> {
    pub fn new() -> Self {
        let mut openapi = OpenAPI::default();
        for flag in inventory::iter::<oasgen_core::SchemaRegister> {
            let schema = (flag.constructor)();
            openapi.schemas.insert(flag.name.to_string(), ReferenceOr::Item(schema));
        }
        // This is required to have stable diffing between builds
        openapi.schemas.sort_keys();
        Self {
            openapi,
            router: Router::default(),
            json_route: None,
            yaml_route: None,
            prefix: None,
            #[cfg(feature = "swagger-ui")]
            swagger_ui_route: None,
            #[cfg(feature = "swagger-ui")]
            swagger_ui: None,
        }
    }

    /// Add a handler to the OpenAPI spec (which is different than mounting it to a server).
    fn add_handler_to_spec<F>(&mut self, path: &str, method: Method, _handler: &F) {
        use http::Method;
        let path = replace_path_params(path);
        let item = self.openapi.paths.paths.entry(path.clone()).or_default();
        let item = item.as_mut().expect("Currently don't support references for PathItem");
        let type_name = std::any::type_name::<F>();
        let mut operation = OPERATION_LOOKUP.get(type_name)

            .expect(&format!("Operation {} not found in OpenAPI spec.", type_name))();
        modify_parameter_names(&mut operation, &path);
        match method {
            Method::GET => item.get = Some(operation),
            Method::POST => item.post = Some(operation),
            Method::PUT => item.put = Some(operation),
            Method::DELETE => item.delete = Some(operation),
            Method::OPTIONS => item.options = Some(operation),
            Method::HEAD => item.head = Some(operation),
            Method::PATCH => item.patch = Some(operation),
            Method::TRACE => item.trace = Some(operation),
            _ => panic!("Unsupported method: {}", method),
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

    /// Configure a prefix to mount the API routes (including the OpenAPI spec routes) under.
    pub fn prefix(mut self, prefix: &str) -> Self {
        self.prefix = Some(prefix.to_string());
        self
    }

    #[cfg(feature = "swagger-ui")]
    #[cfg_attr(docsrs, doc(cfg(feature = "swagger-ui")))]
    /// Specify a path to serve Swagger UI on.
    pub fn swagger_ui(mut self, swagger_ui_route: &str) -> Self {
        if !swagger_ui_route.ends_with('/') {
            panic!("Swagger UI route must end with a slash. Without it, static resources will not be found.");
        }
        let route_without_trailing = &swagger_ui_route[..swagger_ui_route.len() - 1];
        let swagger = swagger_ui::SwaggerUi::default()
            .prefix(route_without_trailing)
            .url(self.json_route.as_ref()
                .or(self.yaml_route.as_ref())
                .expect("Tried to create Swagger UI route, but no JSON or YAML route was set. \
                On `oasgen::Server` instance, call `route_yaml_spec` or `route_json_spec`. \
                If you manually create the route, set the field, call this method, then set the field to None.")
            );
        self.swagger_ui_route = Some(swagger_ui_route.to_string());
        self.swagger_ui = Some(swagger);
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
    pub fn freeze(self) -> Server<Router, Arc<OpenAPI>> {
        Server {
            router: self.router,
            openapi: Arc::new(self.openapi),
            json_route: self.json_route,
            yaml_route: self.yaml_route,
            prefix: self.prefix,
            #[cfg(feature = "swagger-ui")]
            swagger_ui_route: self.swagger_ui_route,
            #[cfg(feature = "swagger-ui")]
            swagger_ui: self.swagger_ui,
        }
    }
}

// Note: this takes an OpenAPI url, which parameterizes like: /path/{param}
fn modify_parameter_names(operation: &mut Operation, path: &str) {
    if !path.contains("{") {
        return;
    }
    let path_parts = path.split("/")
        .filter(|part| part.starts_with("{"))
        .map(|part| &part[1..part.len() - 1]);
    let path_params = operation.parameters.iter_mut()
        .filter_map(|mut p| p.as_mut())
        .filter(|p| matches!(p.kind, ParameterKind::Path { .. }));

    for (part, param) in path_parts.zip(path_params) {
        param.name = part.to_string();
    }
}

// Note: this takes an axum/actix url, which parameterizes like: /path/:param
fn replace_path_params(path: &str) -> String {
    if !path.contains(':') {
        return path.to_string();
    }
    use once_cell::sync::OnceCell;
    use regex::Regex;
    static REMAP: OnceCell<Regex> = OnceCell::new();
    let remap = REMAP.get_or_init(|| Regex::new("/:([a-zA-Z0-9_]+)").unwrap());
    remap.replace_all(&path, "/{$1}").to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use openapiv3 as oa;

    #[test]
    fn test_modify_parameter_names() {
        let path = "/api/v1/pet/{id}/";
        let mut operation = Operation::default();
        operation.parameters.push(Parameter::path("path", oa::Schema::new_number()).into());
        operation.parameters.push(Parameter::query("query", oa::Schema::new_number()).into());
        modify_parameter_names(&mut operation, path);
        assert_eq!(operation.parameters[0].as_item().unwrap().name, "id", "path param name is updated");
        assert_eq!(operation.parameters[1].as_item().unwrap().name, "query", "leave query param alone");
    }

    #[test]
    fn test_replace_path_params() {
        let path = "/api/v1/pet/:id/";
        let path = replace_path_params(path);
        assert_eq!(path, "/api/v1/pet/{id}/");

        let path = "/api/v1/pet/:id";
        let path = replace_path_params(path);
        assert_eq!(path, "/api/v1/pet/{id}");
    }
}
