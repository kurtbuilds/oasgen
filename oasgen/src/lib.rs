#![cfg_attr(docsrs, feature(doc_cfg))]
#![allow(unused)]

mod format;
mod server;

pub use format::*;
pub use oasgen_core::*;
pub use oasgen_macro::{oasgen, OaSchema};
pub use server::Server;

#[cfg(feature = "actix")]
pub use server::actix;

#[cfg(feature = "swagger-ui")]
#[cfg_attr(docsrs, doc(cfg(feature = "swagger-ui")))]
pub use swagger_ui;

pub mod __private {
    pub use inventory;
    pub use oasgen_core::{OperationRegister, SchemaRegister};

    pub fn fn_path_to_op_id(type_name: &str) -> Option<String> {
        Some(type_name.split("::").skip(1).collect::<Vec<_>>().join("_"))
    }
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

#[macro_export]
macro_rules! register_operation {
    ($name:expr, $constructor:expr) => {
        ::oasgen::__private::inventory::submit!(::oasgen::__private::OperationRegister {
            name: $name,
            constructor: &$constructor,
        });
    };
}

/// Use this function if you just want the OpenAPI spec and don't need the server machinery.
/// Note the server machinery is what registers the operations, so this schema only contains
/// the schemas.
pub fn generate_openapi() -> OpenAPI {
    let mut openapi = OpenAPI::default();
    for flag in inventory::iter::<oasgen_core::SchemaRegister> {
        let schema = (flag.constructor)();
        openapi
            .schemas
            .insert(flag.name.to_string(), ReferenceOr::Item(schema));
    }
    // This is required to have stable diffing between builds
    openapi.schemas.sort_keys();
    openapi
}
