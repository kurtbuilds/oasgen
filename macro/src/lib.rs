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
use syn::{visit::Visit};
use axum::http::StatusCode;
use std::collections::HashMap;

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

    //Collect errors from the function AST
    let mut collector = ErrorCollector::new();
    collector.visit_block(&ast.block);
    let mut errors_by_code: HashMap<StatusCode, Vec<String>> = HashMap::new();
    for (status, msg) in &collector.errors {
        errors_by_code.entry(*status).or_default().push(msg.clone());
    }

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
            let body = <#t as ::oasgen::OaParameter>::body_schema();
            if body.is_some() {
                op.add_request_body_json(body);
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
            let body = <#t as ::oasgen::OaParameter>::body_schema();
            if body.is_some() {
                op.add_response_success_json(body);
            }
        }
    }).unwrap_or_default();
    let error_responses = errors_by_code.iter().map(|(status, messages)| {
        let code = status.as_u16();
        let description = if messages.len() == 1 {
            messages[0].clone()
        } else {
            format!("Possible reasons:\n- {}", messages.join("\n- "))
        };
    
        quote! {
            op.add_response_error_json(
                ::oasgen::StatusCode::Code(#code),
                #description.to_string()
            );
        }
    });
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

struct ErrorCollector {
    errors: Vec<(StatusCode, String)>,
    parent_stack: Vec<String>,
    
}
impl ErrorCollector {
    pub fn new() -> Self {
        Self {
            errors: Vec::new(),
            parent_stack: Vec::new(),
        }
    }

    fn extract_error_from_tuple(&mut self, tuple: &syn::ExprTuple) {
        if tuple.elems.len() == 2 {
            if let syn::Expr::Path(status_path) = &tuple.elems[0] {
                let status_code = status_path
                    .path
                    .segments
                    .last()
                    .unwrap()
                    .ident
                    .to_string();
    
                let message = extract_message(&tuple.elems[1]);
                let status_enum = status_code_from_str(&status_code);
    
                self.errors.push((status_enum, message));
            }
        }
    }

    fn extract_formatted_message(&self, mac: &syn::ExprMacro) -> String {
        if mac.mac.path.is_ident("format") {
            let tokens = mac.mac.tokens.to_string();
    
            if let Some(first_literal) = tokens.split(',').next() {
                let fmt_str = first_literal.trim().trim_matches('"');
    
                // Replace `{}` placeholders with {arg_name}
                let mut formatted = fmt_str.to_string();
    
                let args: Vec<&str> = tokens.split(',').skip(1).map(|s| s.trim()).collect();
                for arg in args {
                    // Use the argument name if possible
                    let var_name = arg.split_whitespace().last().unwrap_or("");
                    formatted = formatted.replacen("{}", &format!("{{{}}}", var_name), 1);
                }
    
                return formatted;
            }
        }
        String::new()
        
    }
    
}

fn status_code_from_str(code: &str) -> StatusCode {
    match code {
        "BAD_REQUEST" => StatusCode::BAD_REQUEST,
        "NOT_FOUND" => StatusCode::NOT_FOUND,
        "INTERNAL_SERVER_ERROR" => StatusCode::INTERNAL_SERVER_ERROR,
        "FORBIDDEN" => StatusCode::FORBIDDEN,
        "OK" => StatusCode::OK,
        // add more variants 
        _ => StatusCode::INTERNAL_SERVER_ERROR, // fallback
    }
}


impl<'ast> Visit<'ast> for ErrorCollector {
    fn visit_expr(&mut self, expr: &'ast syn::Expr) {
        // Skip debug! macros
        if let syn::Expr::Macro(macro_expr) = expr {
            let name = macro_expr.mac.path.segments.last().unwrap().ident.to_string();
            if name == "debug" {
                return;
            }
        }

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
                if self.parent_stack.last().map(|s| s == "map_err").unwrap_or(false) {
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







// Helper function to extract string message from an Expr
fn extract_message(expr: &syn::Expr) -> String {
    match expr {
        // Case: direct string literal
        syn::Expr::Lit(syn::ExprLit { lit: syn::Lit::Str(lit_str), .. }) => {
            lit_str.value()
        }

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
                    formatted = formatted.replacen("{}", &format!("{{{}}}", var_name), 1);
                }
    
                return formatted;
            };
            String::new()
        }

        // Case: method call like `err.to_string()`
        syn::Expr::MethodCall(method) if method.method == "to_string" => {
            if let syn::Expr::Lit(syn::ExprLit { lit: syn::Lit::Str(lit_str), .. }) = &*method.receiver {
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
    use quote::ToTokens;
    use super::*;

    #[test]
    fn test_pathed_ty() {
        let ty = syn::parse_str::<Type>("axum::Json<SendCode>").unwrap();
        let ty = turbofish(ty);
        assert_eq!(ty.to_token_stream().to_string(), "axum :: Json :: < SendCode >");
    }
}
