#![cfg_attr(docsrs, feature(doc_cfg))]

#![allow(unused)]
mod server;
mod format;

pub use openapiv3::*;
pub use format::*;
pub use oasgen_macro::{OaSchema, openapi};
pub use oasgen_core::{OaSchema, TypedResponseFuture, FunctionMetadata};
pub use oasgen_core as core;
pub use server::Server;

#[cfg(feature = "swagger-ui")]
#[cfg_attr(docsrs, doc(cfg(feature = "swagger-ui")))]
pub use swagger_ui;

pub mod __private {
    pub use inventory;
    pub use oasgen_core::{SchemaRegister, OperationRegister};
}

pub fn build_schema() -> OpenAPI {
    let mut s = OpenAPI::default();
    let c = s.components_mut();

    for flag in inventory::iter::<oasgen_core::SchemaRegister> {
        let schema = (flag.constructor)();
        c.schemas.insert(flag.name.to_string(), ReferenceOr::Item(schema));
    }
    s
}

pub fn build_yaml() -> String {
    let s = build_schema();
    serde_yaml::to_string(&s).unwrap()
}

#[macro_export]
macro_rules! register_schema {
    ($name:literal, $constructor:expr) => {
        ::oasgen::__private::inventory::submit!(::oasgen::__private::SchemaRegister {
            name: $name,
            constructor: &$constructor,
        });
    };
}