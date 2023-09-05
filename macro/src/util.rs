use oasgen_core::OpenApiAttributes;
use proc_macro::TokenStream;
use quote::quote;
use syn::{punctuated::Punctuated, token::Comma, *};

/// Create OaSchema derive token stream for a struct from ident and fields
pub fn derive_oaschema_struct(ident: &Ident, fields: &Punctuated<Field, Comma>) -> TokenStream {
    let fields: Vec<(&syn::Field, OpenApiAttributes)> = fields
        .into_iter()
        .map(|f| (f, OpenApiAttributes::try_from(&f.attrs).unwrap()))
        .collect::<Vec<_>>();

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

    let name = ident.to_string();
    let ref_name = format!("#/components/schemas/{}", ident);
    let expanded = quote! {
        impl ::oasgen::OaSchema for #ident {
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

/// Create OaSchema derive token stream for a newtype struct from ident and a single inner field
pub fn derive_oaschema_newtype(ident: &Ident, field: &Field) -> TokenStream {
    let ty = &field.ty;
    let name = ident.to_string();
    let expanded = quote! {
        impl ::oasgen::OaSchema for #ident {
            fn schema_name() -> Option<&'static str> {
                Some(<#ty as OaSchema>::schema_name().expect(concat!("No schema name found for ", #name)))
            }

            fn schema_ref() -> Option<::oasgen::ReferenceOr<::oasgen::Schema>> {
                Some(<#ty as OaSchema>::schema_ref().expect(concat!("No schema ref found for ", #name)))
            }

            fn schema() -> Option<::oasgen::Schema> {
                Some(<#ty as OaSchema>::schema().expect(concat!("No schema found for ", #name)))
            }
        }
    };
    TokenStream::from(expanded)
}
