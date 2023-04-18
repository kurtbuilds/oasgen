#![allow(non_snake_case)]

use proc_macro::TokenStream;
use syn::{parse_macro_input, DeriveInput, ReturnType, Token};
use quote::{quote};
use oasgen_core::OpenApiAttributes;

mod util;

#[proc_macro_derive(OaSchema, attributes(openapi))]
pub fn derive_oaschema(item: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(item as DeriveInput);
    let id = &ast.ident;
    let fields = util::get_fields(&ast);

    let fields: Vec<(&syn::Field, OpenApiAttributes)> = fields.into_iter().map(|f| {
        (f, OpenApiAttributes::try_from(&f.attrs).unwrap())
    }).collect::<Vec<_>>();

    let properties = fields.iter().map(|(f, attr)| {
        if attr.skip {
            return quote! {};
        }
        let name = f.ident.as_ref().unwrap().to_string();
        let ty = &f.ty;
        quote! {
            o.add_property(#name, <#ty as OaSchema>::schema().expect(concat!("No schema found for ", #name))).unwrap();
        }
    });

    let required = fields.iter().map(|(f, attr)| {
        if attr.skip || attr.skip_serializing_if.is_some() {
            return quote! {};
        }
        let name = f.ident.as_ref().unwrap().to_string();
        quote! { #name.to_string(), }
    });
    let required = quote! { vec! [ #(#required)* ] };

    let name = id.to_string();
    let ref_name = format!("#/components/schemas/{}", id);
    let expanded = quote! {
        impl ::oasgen::OaSchema for #id {
            fn schema_name() -> Option<&'static str> {
                Some(#name)
            }

            fn schema_ref() -> Option<::oasgen::ReferenceOr<::oasgen::Schema>> {
                Some(::oasgen::ReferenceOr::ref_(#ref_name))
            }

            fn schema() -> Option<::oasgen::Schema> {
                let mut o = ::oasgen::Schema::new_object();
                #(#properties)*
                let req = o.required_mut().unwrap();
                *req = #required;
                Some(o)
            }
        }
    };
    TokenStream::from(expanded)
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
    ast.block = Box::new(syn::parse2(quote!({
        ::oasgen::TypedResponseFuture::new(async move #block)
    })).expect("parsing empty block"));

    let public = ast.vis.clone();

    let bounds = ast.sig.inputs.iter().map(|input| {
        let ty = match input {
            syn::FnArg::Receiver(_) => panic!("receiver not supported"),
            syn::FnArg::Typed(ty) => &ty.ty,
        };
        quote! { #ty: ::oasgen::OaSchema }
    });

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
        #ast

        #[allow(non_camel_case_types)]
        #public struct #marker_struct_name;

        #marker_struct_impl_FunctionMetadata
    };
    TokenStream::from(expanded)
}
