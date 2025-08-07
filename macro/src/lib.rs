#![allow(non_snake_case)]

use proc_macro::TokenStream;
use quote::quote;
use serde_derive_internals::{
    ast::{Container, Data, Style},
    Ctxt, Derive,
};
use syn::{PathArguments, GenericArgument, TypePath, Type, ReturnType, FnArg, parse_macro_input, DeriveInput,visit::Visit};
use std::collections::HashMap;
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
        Data::Struct(Style::Struct, fields) => derive_oaschema_struct(id, fields, docstring),
        Data::Struct(Style::Newtype, fields) => {
            derive_oaschema_newtype(id, fields.first().unwrap())
        }
        Data::Enum(variants) => derive_oaschema_enum(id, variants, &cont.attrs.tag(), docstring),
        Data::Struct(Style::Tuple | Style::Unit, _) => {
            panic!("#[derive(OaSchema)] can not be used on tuple structs")
        }
    }
}

#[proc_macro_attribute]
pub fn oasgen(attr: TokenStream, input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as syn::ItemFn);

    let mut collector = ErrorCollector::default();
    collector.visit_block(&ast.block);

    let mut errors_by_code: HashMap<String, Vec<String>> = HashMap::new();

    for (status_tokens, message) in &collector.errors {
        let key = status_tokens.to_string();
        errors_by_code.entry(key).or_default().push(message.clone());
    }

    let mut attr =
        syn::parse::<OperationAttributes>(attr).expect("Failed to parse operation attributes");
    attr.merge_attributes(&ast.attrs);
    let args = ast
        .sig
        .inputs
        .iter()
        .map(|arg| match arg {
            FnArg::Receiver(_) => panic!("Receiver arguments are not supported"),
            FnArg::Typed(pat) => turbofish(pat.ty.as_ref().clone()),
        })
        .collect::<Vec<_>>();
    let ret = match &ast.sig.output {
        ReturnType::Default => None,
        ReturnType::Type(_, ty) => Some(turbofish(ty.as_ref().clone())),
    };
    let body = args
        .last()
        .map(|t| {
            quote! {
                let body = <#t as ::oasgen::OaParameter>::body_schema();
                if body.is_some() {
                    op.add_request_body_json(body);
                }
            }
        })
        .unwrap_or_default();
    let description = attr
        .description
        .as_ref()
        .map(|s| s.value())
        .map(|c| {
            quote! {
                op.description = Some(#c.to_string());
            }
        })
        .unwrap_or_default();
    let ret = ret
        .map(|t| {
            quote! {
                let body = <#t as ::oasgen::OaParameter>::body_schema();
                if body.is_some() {
                    op.add_response_success_json(body);
                }
            }
        })
        .unwrap_or_default();
    let error_responses: Vec<_> = errors_by_code
        .iter()
        .map(|(status_str, messages)| {
            let status_tokens: TokenStream = status_str.parse().expect("Invalid tokens");

            let description = if messages.len() == 1 {
                messages[0].clone()
            } else {
                format!("Possible reasons:\n- {}", messages.join("\n- "))
            };
            let status_tokens_2: proc_macro2::TokenStream = status_tokens.clone().into();
            quote! {
                op.add_response_error_json(
                    #status_tokens_2.as_u16(),
                    #description.to_string()
                );
            }
        })
        .collect();
    let tags = attr
        .tags
        .iter()
        .flatten()
        .map(|s| {
            quote! {
                op.tags.push(#s.to_string());
            }
        })
        .collect::<Vec<_>>();
    let summary = attr
        .summary
        .as_ref()
        .map(|s| s.value())
        .map(|c| {
            quote! {
                op.summary = Some(#c.to_string());
            }
        })
        .unwrap_or_default();
    let name = ast.sig.ident.to_string();
    let deprecated = attr.deprecated;
    let operation_id = if let Some(id) = attr.operation_id {
        let id = id.value();
        quote! {
            Some(#id.to_string())
        }
    } else {
        quote! {
            ::oasgen::__private::fn_path_to_op_id(concat!(module_path!(), "::", #name))
        }
    };
    let submit = quote! {
        ::oasgen::register_operation!(concat!(module_path!(), "::", #name), || {
            let parameters: Vec<Vec<::oasgen::RefOr<::oasgen::Parameter>>> = vec![
                #( <#args as ::oasgen::OaParameter>::parameters(), )*
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
            #(#error_responses)*
            #(#tags)*
            op
        });
    };
    quote! {
        #ast
        #submit
    }
    .into()
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

#[derive(Default)]
struct ErrorCollector {
    errors: Vec<(proc_macro2::TokenStream, String)>,
    parent_stack: Vec<String>,
}
impl ErrorCollector {
    fn extract_error_from_tuple(&mut self, tuple: &syn::ExprTuple) {
        if tuple.elems.len() == 2 {
            if let syn::Expr::Path(status_path) = &tuple.elems[0] {
                let status_tokens = quote! { #status_path };
                let message = extract_message(&tuple.elems[1]);
                self.errors.push((status_tokens, message));
            }
        }
    }
}

impl<'ast> Visit<'ast> for ErrorCollector {
    fn visit_expr(&mut self, expr: &'ast syn::Expr) {
        // Push the method name if it's a method call (like map_err)
        if let syn::Expr::MethodCall(method_call) = expr {
            self.parent_stack.push(method_call.method.to_string());
        }

        // Visit nested expressions
        syn::visit::visit_expr(self, expr);

        // Now match expressions
        match expr {
            // Handle return Err(...)
            syn::Expr::Return(return_expr) => {
                if let Some(inner) = &return_expr.expr {
                    if let syn::Expr::Call(call) = &**inner {
                        if let syn::Expr::Path(path) = &*call.func {
                            if path.path.is_ident("Err") {
                                if let Some(arg) = call.args.first() {
                                    if let syn::Expr::Tuple(tuple) = arg {
                                        self.extract_error_from_tuple(tuple);
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // Handle tuples, but only if parent is map_err
            syn::Expr::Tuple(tuple) => {
                if self
                    .parent_stack
                    .last()
                    .map(|s| s == "map_err")
                    .unwrap_or(false)
                {
                    self.extract_error_from_tuple(tuple);
                }
            }
            _ => {}
        }

        // Pop parent stack if we pushed
        if let syn::Expr::MethodCall(_) = expr {
            self.parent_stack.pop();
        }
    }
}

/// Helper function to extract string message from an Expr
fn extract_message(expr: &syn::Expr) -> String {
    match expr {
        // Case: direct string literal
        syn::Expr::Lit(syn::ExprLit {
            lit: syn::Lit::Str(lit_str),
            ..
        }) => lit_str.value(),

        // Case: format!(...) macro
        syn::Expr::Macro(mac) if mac.mac.path.is_ident("format") => {
            let tokens = mac.mac.tokens.to_string();

            if let Some(first_literal) = tokens.split(',').next() {
                let fmt_str = first_literal.trim().trim_matches('"');

                // Replace `{}` placeholders with {arg_name}
                let mut formatted = fmt_str.to_string();

                let args: Vec<&str> = tokens.split(',').skip(1).map(|s| s.trim()).collect();
                for arg in args {
                    // Use the argument name if possible
                    let var_name = arg.split_whitespace().last().unwrap_or("");
                    formatted = formatted.replacen("{}", &format!("{{{var_name}}}"), 1);
                }

                return formatted;
            };
            String::new()
        }

        // Case: method call like `err.to_string()`
        syn::Expr::MethodCall(method) if method.method == "to_string" => {
            if let syn::Expr::Lit(syn::ExprLit {
                lit: syn::Lit::Str(lit_str),
                ..
            }) = &*method.receiver
            {
                lit_str.value()
            } else {
                String::new()
            }
        }

        _ => String::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use quote::ToTokens;

    #[test]
    fn test_pathed_ty() {
        let ty = syn::parse_str::<Type>("axum::Json<SendCode>").unwrap();
        let ty = turbofish(ty);
        assert_eq!(
            ty.to_token_stream().to_string(),
            "axum :: Json :: < SendCode >"
        );
    }
}
