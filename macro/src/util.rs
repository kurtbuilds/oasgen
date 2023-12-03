use crate::attr::FieldAttributes;
use proc_macro::TokenStream;
use proc_macro2::{Ident, TokenStream as TokenStream2};
use quote::quote;
use serde_derive_internals::{
    ast::{Field, Variant},
    attr::TagType,
};

pub fn impl_OaSchema_schema(fields: &[Field], docstring: Option<String>) -> TokenStream2 {
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
    let description = docstring.map(|s| {
        quote! {
            o.schema_data.description = Some(#s.into());
        }
    }).unwrap_or_default();
    let properties = fields
        .into_iter()
        .map(|f| {
            let mut attr = FieldAttributes::try_from(&f.original.attrs).unwrap();
            attr.merge_serde(&f);
            if attr.skip {
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
    quote! {
        {
            let mut o = ::oasgen::Schema::new_object();
            #description
            #(#properties)*
            o
        }
    }
}

/// Create OaSchema derive token stream for a struct from ident and fields
pub fn derive_oaschema_struct(ident: &Ident, fields: &[Field], docstring: Option<String>) -> TokenStream {
    let schema = impl_OaSchema_schema(fields, docstring);
    let name = ident.to_string();
    let submit = quote! {
        ::oasgen::register_schema!(#name, || <#ident as ::oasgen::OaSchema>::schema().unwrap());
    };

    quote! {
        impl ::oasgen::OaSchema for #ident {
            fn schema_ref() -> Option<::oasgen::ReferenceOr<::oasgen::Schema>> {
                Some(::oasgen::ReferenceOr::schema_ref(#name))
            }

            fn schema() -> Option<::oasgen::Schema> {
                Some(#schema)
            }
        }
        #submit
    }.into()
}

/// Create OaSchema derive token stream for an enum from ident and variants
pub fn derive_oaschema_enum(ident: &Ident, variants: &[Variant], tag: &TagType, _docstring: Option<String>) -> TokenStream {
    let variants = variants
        .into_iter()
        .filter(|v| {
            let openapi_attrs = FieldAttributes::try_from(&v.original.attrs).unwrap();
            !openapi_attrs.skip
        });
    let mut complex_variants = vec![];
    let mut str_variants = vec![];
    for v in variants {
        let name = v.attrs.name().deserialize_name();
        if v.fields.len() == 0 {
            str_variants.push(quote! { #name.to_string(), });
        } else {
            let schema = impl_OaSchema_schema(&v.fields, None);
            let variant = match tag {
                TagType::External => quote! {
                    {
                        let mut o = ::oasgen::Schema::new_object();
                        o.add_property(#name, ::oasgen::ReferenceOr::Item(#schema)).unwrap();
                        o.required_mut().unwrap().push(#name.to_string());
                        o
                    }
                },
                TagType::Internal { tag } => quote! {
                    {
                        let mut o = #schema;
                        match o.schema_kind {
                            ::oasgen::SchemaKind::Type(_) => {
                                o.add_property(#tag, ::oasgen::ReferenceOr::Item(::oasgen::Schema::new_str_enum(vec![#name.to_string()]))).unwrap();
                                o.required_mut().unwrap().push(#tag.to_string());
                                o
                            }
                            _ => {
                                let mut t = ::oasgen::Schema::new_object();
                                t.add_property(#tag, ::oasgen::ReferenceOr::Item(::oasgen::Schema::new_str_enum(vec![#name.to_string()]))).unwrap();
                                t.required_mut().unwrap().push(#tag.to_string());

                                ::oasgen::Schema {
                                    schema_data: ::oasgen::SchemaData::default(),
                                    schema_kind: ::oasgen::SchemaKind::AllOf {
                                        all_of: vec![
                                            ::oasgen::ReferenceOr::Item(t),
                                            ::oasgen::ReferenceOr::Item(o)
                                        ]
                                    }
                                }
                            }
                        }
                    }
                },
                TagType::Adjacent { tag, content } => quote! {
                    {
                        let mut o = ::oasgen::Schema::new_object();
                        o.add_property(#tag, ::oasgen::ReferenceOr::Item(::oasgen::Schema::new_str_enum(vec![#name.to_string()]))).unwrap();
                        o.add_property(#content, ::oasgen::ReferenceOr::Item(#schema)).unwrap();
                        o.required_mut().unwrap().push(#tag.to_string());
                        o.required_mut().unwrap().push(#content.to_string());
                        o
                    }
                },
                TagType::None => schema,
            };
            complex_variants.push(variant);
        }
    }

    if str_variants.len() > 0 {
        match tag {
            TagType::External => complex_variants.push(quote! { ::oasgen::Schema::new_str_enum(vec![#(#str_variants)*]) }),
            TagType::Internal { tag } | TagType::Adjacent { tag, .. } => complex_variants.push(quote! {{
                let mut o = ::oasgen::Schema::new_object();
                o.add_property(#tag, ::oasgen::ReferenceOr::Item(::oasgen::Schema::new_str_enum(vec![#(#str_variants)*]))).unwrap();
                o.required_mut().unwrap().push(#tag.to_string());
                o
            }}),
            _ => () // a null case should be handled, which will deserialize to the first unit
            // variant, but unsure how to handle this case. I tried an enum with
            // type: 'null', which is supported in glademiller:openapiv3, but not in
            // kurtbuilds:openapiv3
            // kurt: I believe null enum is handled by setting nullable: true.
        }
    }

    let schema = if complex_variants.len() == 1 {
        complex_variants.pop().unwrap()
    } else {
        quote! {
            ::oasgen::Schema {
                schema_data: ::oasgen::SchemaData::default(),
                schema_kind: ::oasgen::SchemaKind::OneOf {
                    one_of: vec![#(::oasgen::ReferenceOr::Item(#complex_variants)),*]
                }
            }
        }
    };

    let name = ident.to_string();
    let submit = quote! {
        ::oasgen::register_schema!(#name, || <#ident as ::oasgen::OaSchema>::schema().unwrap());
    };
    quote! {
        impl ::oasgen::OaSchema for #ident {
            fn schema_ref() -> Option<::oasgen::ReferenceOr<::oasgen::Schema>> {
                Some(::oasgen::ReferenceOr::schema_ref(#name))
            }

            fn schema() -> Option<::oasgen::Schema> {
                Some(#schema)
            }
        }
        #submit
    }.into()
}

pub fn derive_oaschema_newtype(ident: &Ident, field: &Field) -> TokenStream {
    let ty = &field.ty;
    let name = ident.to_string();
    quote! {
        impl ::oasgen::OaSchema for #ident {
            fn schema_ref() -> Option<::oasgen::ReferenceOr<::oasgen::Schema>> {
                Some(<#ty as OaSchema>::schema_ref().expect(concat!("No schema ref found for ", #name)))
            }

            fn schema() -> Option<::oasgen::Schema> {
                Some(<#ty as OaSchema>::schema().expect(concat!("No schema found for ", #name)))
            }
        }
    }.into()
}