#![allow(non_snake_case)]

use proc_macro::TokenStream;
use quote::quote;
use serde_derive_internals::{
    ast::{Container, Data, Style},
    Ctxt, Derive,
};
use syn::{PathArguments, GenericArgument, TypePath, Type, ReturnType, FnArg, parse_macro_input, DeriveInput};
use util::{derive_oaschema_enum, derive_oaschema_struct};
use crate::attr::{get_docstring, OperationAttributes};
use crate::util::derive_oaschema_newtype;

mod util;
mod attr;

#[proc_macro_derive(OaSchema, attributes(oasgen))]
pub fn derive_oaschema(item: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(item as DeriveInput);

    let cont = {
        let ctxt = Ctxt::new();
        let cont = Container::from_ast(&ctxt, &ast, Derive::Deserialize);
        ctxt.check().unwrap();
        cont.unwrap()
    };

    let id = &cont.ident;
    let docstring = get_docstring(&ast.attrs).expect("Failed to parse docstring");
    match &cont.data {
        Data::Struct(Style::Struct, fields) => {
            derive_oaschema_struct(id, fields, docstring)
        }
        Data::Struct(Style::Newtype, fields) => {
            derive_oaschema_newtype(id, fields.first().unwrap())
        }
        Data::Enum(variants) => {
            derive_oaschema_enum(id, variants, &cont.attrs.tag(), docstring)
        }
        Data::Struct(Style::Tuple | Style::Unit, _) => {
            panic!("#[derive(OaSchema)] can not be used on tuple structs")
        }
    }
}


#[proc_macro_attribute]
pub fn oasgen(attr: TokenStream, input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as syn::ItemFn);
    let mut attr = syn::parse::<OperationAttributes>(attr).expect("Failed to parse operation attributes");
    attr.merge_attributes(&ast.attrs);
    let args = ast.sig.inputs.iter().map(|arg| {
        match arg {
            FnArg::Receiver(_) => panic!("Receiver arguments are not supported"),
            FnArg::Typed(pat) => turbofish(pat.ty.as_ref().clone()),
        }
    }).collect::<Vec<_>>();
    let ret = match &ast.sig.output {
        ReturnType::Default => None,
        ReturnType::Type(_, ty) => Some(turbofish(ty.as_ref().clone())),
    };
    let body = args.last().map(|t| {
        quote! {
            let body = #t::body_schema();
            if body.is_some() {
                op.add_request_body_json(#t::body_schema());
            }
        }
    }).unwrap_or_default();
    let description = attr.description.as_ref().map(|s| s.value()).map(|c| {
        quote! {
            op.description = Some(#c.to_string());
        }
    }).unwrap_or_default();
    let ret = ret.map(|t| {
        quote! {
            let body = #t::body_schema();
            if body.is_some() {
                op.add_response_success_json(body);
            }
        }
    }).unwrap_or_default();
    let tags = attr.tags.iter().flatten().map(|s| {
        quote! {
            op.tags.push(#s.to_string());
        }
    }).collect::<Vec<_>>();
    let summary = attr.summary.as_ref().map(|s| s.value()).map(|c| {
        quote! {
            op.summary = Some(#c.to_string());
        }
    }).unwrap_or_default();
    let name = ast.sig.ident.to_string();
    let deprecated = attr.deprecated;
    let operation_id = if let Some(id) = attr.operation_id {
        let id = id.value();
        quote! {
            op.operation_id = Some(#id.to_string())
        }
    } else {
        quote! {
            ::oasgen::__private::fn_path_to_op_id(concat!(module_path!(), "::", #name))
        }
    };
    let submit = quote! {
        ::oasgen::register_operation!(concat!(module_path!(), "::", #name), || {
            let parameters: Vec<Vec<::oasgen::RefOr<::oasgen::Parameter>>> = vec![
                #( #args::parameters(), )*
            ];
            let parameters = parameters
                .into_iter()
                .flatten()
                .collect::<Vec<::oasgen::RefOr<::oasgen::Parameter>>>();
            let mut op = ::oasgen::Operation::default();
            op.operation_id = #operation_id;
            op.parameters = parameters;
            op.deprecated = #deprecated;
            #body
            #ret
            #description
            #summary
            #(#tags)*
            op
        });
    };
    quote! {
        #ast
        #submit
    }.into()
}

/// insert the turbofish :: into a syn::Type
/// example: axum::Json<User> becomes axum::Json::<User>
fn turbofish(mut ty: Type) -> Type {
    fn inner(ty: &mut Type) {
        match ty {
            Type::Path(TypePath { path, .. }) => {
                let Some(last) = path.segments.last_mut() else {
                    return;
                };
                match &mut last.arguments {
                    PathArguments::AngleBracketed(args) => {
                        args.colon2_token = Some(Default::default());
                        for arg in args.args.iter_mut() {
                            match arg {
                                GenericArgument::Type(ty) => {
                                    inner(ty);
                                }
                                _ => {}
                            }
                        }
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    }
    inner(&mut ty);
    ty
}

#[cfg(test)]
mod tests {
    use quote::ToTokens;
    use super::*;

    #[test]
    fn test_pathed_ty() {
        let ty = syn::parse_str::<Type>("axum::Json<SendCode>").unwrap();
        let ty = turbofish(ty);
        assert_eq!(ty.to_token_stream().to_string(), "axum :: Json :: < SendCode >");
    }
}
