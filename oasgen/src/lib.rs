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