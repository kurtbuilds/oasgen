use proc_macro::TokenStream;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(OaSchema), attributes(openapi)]
pub fn derive_oaschema(item: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(item as DeriveInput);
    let id = ast.ident;
    let ref_name = format!("#/components/schemas/{}", id);
    let expanded = quote! {
        impl opanapi_spec_gen::OaSchema for #id {
            fn schema() -> Some(openapiv3::Schema) {
                Some(openapiv3::Schema::new_object())
            }

            fn schema_ref() -> Some(String) {
                Some(#ref_name.to_string())
            }
        }
    };
    TokenStream::from(expanded)
}