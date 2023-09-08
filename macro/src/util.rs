use oasgen_core::OpenApiAttributes;
use proc_macro::TokenStream;
use proc_macro2::{Ident, TokenStream as TokenStream2};
use quote::quote;
use serde_derive_internals::{
    ast::{Field, Variant},
    attr::TagType,
};

pub fn derive_oaschema_process_fields(fields: &[Field]) -> TokenStream2 {
    // newtype
    if fields.len() == 1 {
        let field = fields.first().unwrap();
        if let syn::Member::Unnamed(_) = field.member {
            let name = field.attrs.name().deserialize_name();
            let ty = field.ty;
            return quote! {
                <#ty as OaSchema>::schema().expect(concat!("No schema found for ", #name))
            };
        }
    }

    let properties = fields
        .into_iter()
        .map(|f| {
            let openapi_attrs = OpenApiAttributes::try_from(&f.original.attrs).unwrap();

            if openapi_attrs.skip {
                return quote! {};
            }

            let name = f.attrs.name().deserialize_name();
            let ty = f.ty;
            let schema = quote! {
                <#ty as OaSchema>::schema().expect(concat!("No schema found for ", #name))
            };

            if f.attrs.flatten() {
                quote! {
                    if let ::oasgen::SchemaKind::Type(::oasgen::Type::Object(::oasgen::ObjectType { properties, required, .. }))
                            = #schema.schema_kind {
                        for (name, schema) in properties.into_iter() {
                            match schema {
                                ::oasgen::ReferenceOr::Item(mut item) => o.add_property(&name, item.clone()).unwrap(),
                                ::oasgen::ReferenceOr::Reference {..} => panic!("Cannot flatten a reference")
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
                    o.add_property(#name, #schema).unwrap();
                    #required
                }
            }
        })
        .collect::<Vec<_>>();
    quote! {
        {
            let mut o = ::oasgen::Schema::new_object();
            #(#properties)*
            o
        }
    }
}

/// Create OaSchema derive token stream for a struct from ident and fields
pub fn derive_oaschema_struct(ident: &Ident, fields: &[Field]) -> TokenStream {
    let schema = derive_oaschema_process_fields(fields);
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
                Some(#schema)
            }
        }
    };
    TokenStream::from(expanded)
}

/// Create OaSchema derive token stream for an enum from ident and variants
pub fn derive_oaschema_enum(ident: &Ident, variants: &[Variant], tag: &TagType) -> TokenStream {
    let (mut complex_variants, str_variants) = variants
        .into_iter()
        .filter(|v| {
            let openapi_attrs = OpenApiAttributes::try_from(&v.original.attrs).unwrap();
            !openapi_attrs.skip
        })
        .fold(
            (vec![], vec![]),
            |(mut complex_variants, mut str_variants), v| {
                let name = v.attrs.name().deserialize_name();

                if v.fields.len() == 0 {
                    str_variants.push(quote! { #name.to_string(), });
                    (complex_variants, str_variants)
                } else {
                    let schema = derive_oaschema_process_fields(&v.fields);
                    let variant = match tag {
                        TagType::External => quote! {{
                            let mut o = ::oasgen::Schema::new_object();
                            o.add_property(#name, #schema).unwrap();
                            o.required_mut().unwrap().push(#name.to_string());
                            o
                        }},
                        TagType::Internal { tag } => quote! {{
                            let mut o = #schema;
                            o.add_property(#tag, ::oasgen::Schema::new_str_enum(vec![#name.to_string()])).unwrap();
                            o.required_mut().unwrap().push(#tag.to_string());
                            o
                        }},
                        TagType::Adjacent { tag, content } => quote! {{
                            let mut o = ::oasgen::Schema::new_object();
                            o.add_property(#tag, ::oasgen::Schema::new_str_enum(vec![#name.to_string()])).unwrap();
                            o.add_property(#content, #schema).unwrap();
                            o.required_mut().unwrap().push(#tag.to_string());
                            o.required_mut().unwrap().push(#content.to_string());
                            o
                        }},
                        TagType::None => schema,
                    };
                    complex_variants.push(variant);
                    (complex_variants, str_variants)
                }
            },
        );

    if str_variants.len() > 0 {
        match tag {
            TagType::External => complex_variants.push(quote! { ::oasgen::Schema::new_str_enum(vec![#(#str_variants)*]) }),
            TagType::Internal { tag } => complex_variants.push(quote! {{
                let mut o = ::oasgen::Schema::new_object();
                o.add_property(#tag, ::oasgen::Schema::new_str_enum(vec![#(#str_variants)*])).unwrap();
                o.required_mut().unwrap().push(#tag.to_string());
                o
            }}),
            TagType::Adjacent { tag, .. } => complex_variants.push(quote! {{
                let mut o = ::oasgen::Schema::new_object();
                o.add_property(#tag, ::oasgen::Schema::new_str_enum(vec![#(#str_variants)*])).unwrap();
                o.required_mut().unwrap().push(#tag.to_string());
                o
            }}),
            _ => () // a null case should be handled, which will deserialize to the first unit
                    // variant, but unsure how to handle this case. I tried an enum with
                    // type: 'null', which is supported in glademiller:openapiv3, but not in
                    // kurtbuilds:openapiv3
        }
    }

    let schema = if complex_variants.len() == 1 {
        complex_variants.pop().unwrap()
    } else {
        quote! { ::oasgen::Schema {
            schema_data: ::oasgen::SchemaData::default(),
            schema_kind: ::oasgen::SchemaKind::OneOf {
                one_of: vec![#(::oasgen::ReferenceOr::item(#complex_variants)),*]
            }
        } }
    };

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
                Some(#schema)
            }
        }
    };

    TokenStream::from(expanded)
}
