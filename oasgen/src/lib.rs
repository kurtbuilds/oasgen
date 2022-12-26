#![allow(unused)]
mod server;

pub use openapiv3::*;
pub use oasgen_macro::{OaSchema, openapi};
pub use oasgen_core::{OaSchema, TypedResponseFuture, FunctionMetadata};
pub use oasgen_core as core;
pub use server::Server;

#[cfg(test)]
mod tests {
    // use super::*;

    #[test]
    fn it_works() {
    }
}
