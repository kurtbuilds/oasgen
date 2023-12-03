use quote::ToTokens;
use structmeta::StructMeta;
use syn::LitStr;
use serde_derive_internals::ast::{Field};

/// Available attributes on a struct
/// For attributes that have the same name as `serde` attributes, you can use either one.
/// For example, `skip` will be applied with either `#[openapi(skip)]` or `#[serde(skip)]`.
#[derive(StructMeta, Default)]
pub struct FieldAttributes {
    pub skip: bool,
    pub skip_serializing_if: Option<LitStr>,
    pub inline: bool,
}

impl FieldAttributes {
    pub fn merge_with(&mut self, other: &Self) {
        if other.skip {
            self.skip = true;
        }
        if other.inline {
            self.inline = true;
        }
        if other.skip_serializing_if.is_some() {
            self.skip_serializing_if = other.skip_serializing_if.clone();
        }
    }

    pub fn merge_serde(&mut self, other: &Field) {
        if other.attrs.skip_serializing() {
            self.skip = true;
        }
        if let Some(skip_serializing_if) = other.attrs.skip_serializing_if() {
            self.skip_serializing_if = Some(LitStr::new(&skip_serializing_if.to_token_stream().to_string(), proc_macro2::Span::call_site()));
        }
    }
}

impl TryFrom<&Vec<syn::Attribute>> for FieldAttributes {
    type Error = syn::Error;

    fn try_from(attrs: &Vec<syn::Attribute>) -> Result<Self, Self::Error> {
        let attrs = attrs.into_iter()
            .filter(|a| a.path().get_ident().map(|i| i == "openapi").unwrap_or(false))
            .map(|a| a.parse_args())
            .collect::<Result<Vec<FieldAttributes>, syn::Error>>()?;
        let mut attrs = attrs.into_iter();
        let mut result = attrs.next().unwrap_or_default();
        for attr in attrs {
            result.merge_with(&attr);
        }
        Ok(result)
    }
}