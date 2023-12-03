use http::Method;
use oasgen_core::{OaSchema};
use crate::Server;


impl Server<()> {
    pub fn none() -> Self {
        Self::new()
    }

    pub fn get<F>(mut self, path: &str, handler: F) -> Self {
        self.add_handler_to_spec(path, Method::GET, &handler);
        self
    }

    pub fn post<F>(mut self, path: &str, handler: F) -> Self {
        self.add_handler_to_spec(path, Method::POST, &handler);
        self
    }
}