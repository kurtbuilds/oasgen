use proc_macro::TokenStream;
use syn::{parse_macro_input, DeriveInput};
use quote::quote;
mod util;

#[proc_macro_derive(OaSchema, attributes(openapi))]
pub fn derive_oaschema(item: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(item as DeriveInput);
    let id = &ast.ident;
    let fields = util::get_fields(&ast);

    let properties = fields.into_iter().map(|f| {
        let name = f.ident.as_ref().unwrap().to_string();
        let ty = &f.ty;
        quote! {
            o.add_property(#name, <#ty as OaSchema>::schema().unwrap());
        }
    });

    let name = id.to_string();
    let ref_name = format!("#/components/schemas/{}", id);
    let expanded = quote! {
        impl oasgen::OaSchema for #id {
            fn name() -> Option<&'static str> {
                Some(#name)
            }

            fn schema_ref() -> Option<oasgen::ReferenceOr<oasgen::Schema>> {
                Some(oasgen::ReferenceOr::ref_(#ref_name))
            }

            fn schema() -> Option<oasgen::Schema> {
                let mut o = oasgen::Schema::new_object();
                #(#properties)*
                Some(o)
            }
        }
    };
    TokenStream::from(expanded)
}