use crate::attr::FieldAttributes;
use proc_macro::TokenStream;
use proc_macro2::Ident;
use quote::quote;
use serde_derive_internals::ast::{Field, Variant};

/// Create OaSchema derive token stream for a struct from ident and fields
pub fn derive_oaschema_struct(ident: &Ident, fields: &[Field]) -> TokenStream {
    let properties = fields
        .into_iter()
        .map(|f| {
            let mut attr = FieldAttributes::try_from(&f.original.attrs).unwrap();
            attr.merge_serde(&f);
            if attr.skip || f.attrs.skip_serializing() {
                return quote! {};
            }

            let name = f.attrs.name().deserialize_name();
            let ty = f.ty;
            let schema = quote! {
                <#ty as ::oasgen::OaSchema>::schema().expect(concat!("No schema found for ", #name))
            };

            if f.attrs.flatten() {
                quote! {
                    if let ::oasgen::SchemaKind::Type(::oasgen::Type::Object(::oasgen::ObjectType { properties, required, .. }))
                            = #schema.schema_kind {
                        for (name, schema) in properties.into_iter() {
                            let schema = schema.into_item().expect("Cannot flatten a reference");
                            o.add_property(&name, ::oasgen::ReferenceOr::Item(schema)).unwrap();
                        }
                        o.required_mut().unwrap().extend_from_slice(&required);
                    }
                }
            } else {
                let required = if attr.skip || attr.skip_serializing_if.is_some() {
                    quote! {}
                } else {
                    quote! { o.required_mut().unwrap().push(#name.to_string()); }
                };
                let schema_ref = if attr.inline {
                    quote! {
                        ::oasgen::ReferenceOr::Item(<#ty as ::oasgen::OaSchema>::schema().expect(concat!("No schema ref found for ", #name)))
                    }
                } else {
                    quote! {
                        <#ty as ::oasgen::OaSchema>::schema_ref().expect(concat!("No schema ref found for ", #name))
                    }
                };
                quote! {
                    o.add_property(#name, #schema_ref).unwrap();
                    #required
                }
            }
        })
        .collect::<Vec<_>>();

    let name = ident.to_string();
    let submit = quote! {
        ::oasgen::register_schema!(#name, || <#ident as ::oasgen::OaSchema>::schema().unwrap());
    };
    quote! {
        impl ::oasgen::OaSchema for #ident {
            fn schema_name() -> Option<&'static str> {
                Some(#name)
            }

            fn schema_ref() -> Option<::oasgen::ReferenceOr<::oasgen::Schema>> {
                Some(::oasgen::ReferenceOr::schema_ref(#name))
            }

            fn schema() -> Option<::oasgen::Schema> {
                let mut o = ::oasgen::Schema::new_object();
                #(#properties)*
                Some(o)
            }
        }
        #submit
    }.into()
}

/// Create OaSchema derive token stream for a newtype struct from ident and a single inner field
pub fn derive_oaschema_newtype(ident: &Ident, field: &Field) -> TokenStream {
    let ty = &field.ty;
    let name = ident.to_string();
    let submit = quote! {
        ::oasgen::register_schema!(#name, || <#ident as ::oasgen::OaSchema>::schema().unwrap());
    };
    quote! {
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
        #submit
    }.into()
}

/// Create OaSchema derive token stream for an enum from ident and variants
pub fn derive_oaschema_enum(ident: &Ident, variants: &[Variant]) -> TokenStream {
    let str_variants = variants
        .into_iter()
        .map(|v| {
            let openapi_attrs = FieldAttributes::try_from(&v.original.attrs).unwrap();

            if openapi_attrs.skip {
                return quote! {};
            }
            assert!(v.fields.len() == 0, "Enum with fields not supported.");
            let name = v.attrs.name().deserialize_name();
            quote! { #name.to_string(), }
        })
        .collect::<Vec<_>>();

    let name = ident.to_string();
    let submit = quote! {
        ::oasgen::register_schema!(#name, || <#ident as ::oasgen::OaSchema>::schema().unwrap());
    };
    quote! {
        impl ::oasgen::OaSchema for #ident {
            fn schema_name() -> Option<&'static str> {
                Some(#name)
            }

            fn schema_ref() -> Option<::oasgen::ReferenceOr<::oasgen::Schema>> {
                Some(::oasgen::ReferenceOr::schema_ref(#name))
            }

            fn schema() -> Option<::oasgen::Schema> {
                Some(::oasgen::Schema::new_str_enum(vec![#(#str_variants)*]))
            }
        }
        #submit
    }.into()
}