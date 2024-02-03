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
            let ty = field.ty;
            return quote! {
                <#ty as OaSchema>::schema()
            };
        }
    }
    let description = docstring.map(|s| {
        quote! {
            o.description = Some(#s.into());
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
                <#ty as ::oasgen::OaSchema>::schema()
            };

            if f.attrs.flatten() {
                quote! {
                    if let ::oasgen::SchemaKind::Type(::oasgen::Type::Object(::oasgen::ObjectType { properties, required, .. })) = #schema.kind {
                        for (name, schema) in properties {
                            let schema = schema.into_item().expect("Cannot flatten a reference");
                            o.properties_mut().insert(name, schema);
                        }
                        o.required_mut().extend_from_slice(&required);
                    }
                }
            } else {
                let required = !(attr.skip || attr.skip_serializing_if.is_some());
                let required = required.then(|| {
                    quote! { o.required_mut().push(#name.to_string()); }
                }).unwrap_or_default();
                let schema_ref = if attr.inline {
                    quote! {
                        <#ty as ::oasgen::OaSchema>::schema()
                    }
                } else {
                    quote! {
                        <#ty as ::oasgen::OaSchema>::schema_ref()
                    }
                };
                quote! {
                    o.properties_mut().insert(#name, #schema_ref);
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
        ::oasgen::register_schema!(#name, || <#ident as ::oasgen::OaSchema>::schema());
    };

    quote! {
        impl ::oasgen::OaSchema for #ident {
            fn schema_ref() -> ::oasgen::ReferenceOr<::oasgen::Schema> {
                ::oasgen::ReferenceOr::schema_ref(#name)
            }

            fn schema() -> ::oasgen::Schema {
                #schema
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
                        o.properties_mut().insert(#name, #schema);
                        o.required_mut().push(#name.to_string());
                        o
                    }
                },
                TagType::Internal { tag } => quote! {
                    {
                        let mut o = #schema;
                        match o.kind {
                            ::oasgen::SchemaKind::Type(_) => {
                                o.properties_mut().insert(#tag, ::oasgen::Schema::new_str_enum(vec![#name.to_string()]));
                                o.required_mut().push(#tag.to_string());
                                o
                            }
                            _ => {
                                let mut t = ::oasgen::Schema::new_object();
                                t.properties_mut().insert(#tag, ::oasgen::Schema::new_str_enum(vec![#name.to_string()]));
                                t.required_mut().push(#tag.to_string());

                                ::oasgen::Schema {
                                    data: ::oasgen::SchemaData::default(),
                                    kind: ::oasgen::SchemaKind::AllOf {
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
                        let values = vec![#name.to_string()];
                        o.properties_mut().insert(#tag, ::oasgen::Schema::new_str_enum(values));
                        o.properties_mut().insert(#content, #schema);
                        let required = o.required_mut();
                        required.push(#tag.to_string());
                        required.push(#content.to_string());
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
                let values = vec![#(#str_variants)*];
                o.properties_mut().insert(#tag, ::oasgen::Schema::new_str_enum(values));
                o.required_mut().push(#tag.to_string());
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
            ::oasgen::Schema::new_one_of(vec![#(::oasgen::ReferenceOr::Item(#complex_variants)),*])
        }
    };

    let name = ident.to_string();
    let submit = quote! {
        ::oasgen::register_schema!(#name, || <#ident as ::oasgen::OaSchema>::schema());
    };
    quote! {
        impl ::oasgen::OaSchema for #ident {
            fn schema_ref() -> ::oasgen::RefOr<::oasgen::Schema> {
                ::oasgen::RefOr::schema_ref(#name)
            }

            fn schema() -> ::oasgen::Schema {
                #schema
            }
        }
        #submit
    }.into()
}

pub fn derive_oaschema_newtype(ident: &Ident, field: &Field) -> TokenStream {
    let ty = &field.ty;
    quote! {
        impl ::oasgen::OaSchema for #ident {
            fn schema_ref() -> ::oasgen::RefOr<::oasgen::Schema> {
                <#ty as OaSchema>::schema_ref()
            }

            fn schema() -> ::oasgen::Schema {
                <#ty as OaSchema>::schema()
            }
        }
    }.into()
}
