use oasgen_core::OpenApiAttributes;
use proc_macro::TokenStream;
use proc_macro2::Ident;
use quote::quote;
use serde_derive_internals::ast::{Field, Variant};

/// Create OaSchema derive token stream for a struct from ident and fields
pub fn derive_oaschema_struct(ident: &Ident, fields: &[Field]) -> TokenStream {
    let properties = fields
        .into_iter()
        .map(|f| {
            let openapi_attrs = OpenApiAttributes::try_from(&f.original.attrs).unwrap();

            if openapi_attrs.skip {
                return quote! {};
            }

            let name = f.attrs.name().deserialize_name();
            let ty = f.ty;

            if f.attrs.flatten() {
                quote! {
                    if let oasgen_core::SchemaKind::Type(oasgen_core::Type::Object(oasgen_core::ObjectType { properties, required, .. }))
                            = <#ty as OaSchema>::schema().expect(concat!("No schema found for ", #name)).schema_kind {
                        for (name, schema) in properties.into_iter() {
                            match schema {
                                oasgen_core::ReferenceOr::Item(mut item) => o.add_property(&name, item.clone()).unwrap(),
                                oasgen_core::ReferenceOr::Reference {..} => panic!("Cannot flatten a reference")
                            }
                        }
                        o.required_mut().unwrap().extend_from_slice(&required);
                    }
                }
            } else {
                let required = if openapi_attrs.skip || openapi_attrs.skip_serializing_if.is_some() {
                    quote! {}
                } else {
                    quote! { o.required_mut().unwrap().push(#name.to_string()); }
                };

                quote! {
                    o.add_property(#name, <#ty as OaSchema>::schema().expect(concat!("No schema found for ", #name))).unwrap();
                    #required
                }
            }
        })
        .collect::<Vec<_>>();

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

/// Create OaSchema derive token stream for an enum from ident and variants
pub fn derive_oaschema_enum(ident: &Ident, variants: &[Variant]) -> TokenStream {
    let variants: Vec<(&Variant, OpenApiAttributes)> = variants
        .into_iter()
        .map(|v| (v, OpenApiAttributes::try_from(&v.original.attrs).unwrap()))
        .collect::<Vec<_>>();

    let str_variants = variants.iter().map(|(v, attr)| {
        if attr.skip {
            return quote! {};
        }
        assert!(v.fields.len() == 0, "Enum with fields not supported.");
        let name = v.attrs.name().deserialize_name();
        quote! { #name.to_string(), }
    });

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
                let mut o = ::oasgen::Schema::new_str_enum(vec![#(#str_variants)*]);
                Some(o)
            }
        }
    };

    TokenStream::from(expanded)
}
