use http::Method;
use oasgen_core::{OaOperation, OaSchema};
use crate::Server;

// We don't need to save the resources for mounting later, so this is a no-op struct.
pub struct InnerResourceFactory<'a> { _marker: std::marker::PhantomData<&'a ()> }

impl<'a> InnerResourceFactory<'a> {
    pub fn manual_clone(&self) -> InnerResourceFactory<'static> {
        InnerResourceFactory { _marker: std::marker::PhantomData }
    }
}

impl Server {
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