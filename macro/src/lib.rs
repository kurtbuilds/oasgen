#![allow(non_snake_case)]

use proc_macro::TokenStream;
use quote::quote;
use serde_derive_internals::{
    ast::{Container, Data, Style},
    Ctxt, Derive,
};
use syn::*;
use util::{derive_oaschema_enum, derive_oaschema_struct};

mod util;

#[proc_macro_derive(OaSchema, attributes(openapi))]
pub fn derive_oaschema(item: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(item as DeriveInput);

    let cont = {
        let ctxt = Ctxt::new();
        let cont = Container::from_ast(&ctxt, &ast, Derive::Deserialize);
        ctxt.check().unwrap();
        cont.unwrap()
    };

    let id = &cont.ident;

    match &cont.data {
        Data::Struct(Style::Tuple, _) => {
            panic!("#[ormlite] can not be used on tuple structs")
        }
        Data::Struct(Style::Unit, _) => {
            panic!("#[ormlite] can not be used on unit structs")
        }
        Data::Struct(_, fields) => derive_oaschema_struct(id, fields),
        Data::Enum(variants) => derive_oaschema_enum(id, variants),
    }
}

#[proc_macro_attribute]
pub fn openapi(_args: TokenStream, input: TokenStream) -> TokenStream {
    let span = proc_macro2::Span::call_site();

    let mut ast = parse_macro_input!(input as syn::ItemFn);
    // println!("{:#?}", _args);
    let name = &ast.sig.ident;
    let marker_struct_name = syn::Ident::new(&format!("__{}__metadata", name), name.span());

    ast.sig.asyncness = None;
    let output_type = match std::mem::replace(&mut ast.sig.output, ReturnType::Default) {
        ReturnType::Type(_, ty) => ty,
        ReturnType::Default => Box::new(syn::parse2(quote!(())).unwrap()),
    };
    ast.sig.output = ReturnType::Type(
        Token![->](span),
        Box::new(syn::parse2(quote!(::oasgen::TypedResponseFuture<impl std::future::Future<Output=#output_type>, #marker_struct_name>)).expect("parsing empty type")),
    );

    let block = &ast.block;
    ast.block = Box::new(
        syn::parse2(quote!({
            ::oasgen::TypedResponseFuture::new(async move #block)
        }))
        .expect("parsing empty block"),
    );

    let public = ast.vis.clone();

    let bounds = ast.sig.inputs.iter().map(|input| {
        let ty = match input {
            syn::FnArg::Receiver(_) => panic!("receiver not supported"),
            syn::FnArg::Typed(ty) => &ty.ty,
        };
        quote! { #ty: ::oasgen::OaSchema }
    });

    // Leaving this code in, because we want to do something like this, but I can't figure out
    // how to deal with the generic S on axum::extract::FromRequest<S>.
    // However, if we figure this out, it'll give much nicer error messages.

    // Right now, if you're missing a FromRequest/FromRequestParts impl on a parameter's type, you'll get an error message like this:
    // --> server/src/bin/server.rs:137:30
    //     |
    //     137 |         .post("/auth/login", auth::login)
    //     |          ----                ^^^^^^^^^^^ the trait `Handler<_, Pool<Postgres>>` is not implemented for fn item `fn(axum::extract::State<Pool<Postgres>>,
    // RequestId, Uri, axum::Json<EmailPassword>) -> TypedResponseFuture<impl std::future::Future<Output = Result<hyper::Response<http_body::combinators::box_body::UnsyncBoxBody<axum::body::Bytes, axum::Error>>, server::prelude::Error>>, __login__metadata> {login}`

    // which is... not great. We'd like to just assert at compile time that all fn arguments 1..n-1 implement FromRequestParts, and argument n implements FromRequest.

    // let axum_bounds = if cfg!(feature = "axum") {
    //     let bounds = ast.sig.inputs.iter().map(|input| {
    //         let ty = match input {
    //             syn::FnArg::Receiver(_) => panic!("receiver not supported"),
    //             syn::FnArg::Typed(ty) => &ty.ty,
    //         };
    //         quote! { impl<S, B> ::oasgen::axum::CompileCheckImplementsExtract<S, B> for #ty {
    //             type S;
    //             type B;
    //         } }
    //     });
    //     quote! {
    //         #(#bounds )*
    //     }
    // } else {
    //     TokenStream2::new()
    // };

    // println!("{}", ast.to_token_stream());
    let marker_struct_impl_FunctionMetadata = quote! {
        impl ::oasgen::FunctionMetadata for #marker_struct_name where
            #output_type: OaSchema
            #(, #bounds )*
        {
            fn operation_id() -> Option<&'static str> {
                None
            }

            fn summary() -> Option<&'static str> {
                None
            }

            fn description() -> Option<&'static str> {
                None
            }
        }
    };
    let expanded = quote! {
        // #axum_bounds

        #ast

        #[allow(non_camel_case_types)]
        #public struct #marker_struct_name;

        #marker_struct_impl_FunctionMetadata
    };
    TokenStream::from(expanded)
}
