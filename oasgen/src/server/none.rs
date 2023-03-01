use http::Method;
use oasgen_core::{OaOperation, OaSchema};
use crate::Server;


impl Server<()> {
    pub fn get<F, Signature>(mut self, path: &str, handler: F) -> Self
        where
            F: OaOperation<Signature>,
    {
        self.add_handler_to_spec(path, Method::GET, &handler);
        self
    }

    pub fn post<F, Signature>(mut self, path: &str, handler: F) -> Self
        where
            F: OaOperation<Signature>
    {
        self.add_handler_to_spec(path, Method::POST, &handler);
        self
    }
}