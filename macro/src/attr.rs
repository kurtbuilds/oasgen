use quote::ToTokens;
use serde_derive_internals::ast::Field;
use structmeta::StructMeta;
use syn::spanned::Spanned;
use syn::{Ident, LitStr};

/// Available attributes on a struct
/// For attributes that have the same name as `serde` attributes, you can use either one.
/// For example, `skip` will be applied with either `#[oasgen(skip)]` or `#[serde(skip)]`.
#[derive(StructMeta, Default)]
pub struct FieldAttributes {
    pub skip: bool,
    pub skip_serializing_if: Option<LitStr>,
    /// By default, oasgen will use references when possible
    /// If you want to inline the schema, use `#[oasgen(inline)]`
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
            self.skip_serializing_if = Some(LitStr::new(
                &skip_serializing_if.to_token_stream().to_string(),
                proc_macro2::Span::call_site(),
            ));
        }
    }
}

impl TryFrom<&Vec<syn::Attribute>> for FieldAttributes {
    type Error = syn::Error;

    fn try_from(attrs: &Vec<syn::Attribute>) -> Result<Self, Self::Error> {
        let attrs = attrs
            .iter()
            .filter(|a| a.path().get_ident().map(|i| i == "oasgen").unwrap_or(false))
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

/// available parameters for #[oasgen] attribute.
#[derive(StructMeta, Default)]
pub struct OperationAttributes {
    pub summary: Option<LitStr>,
    pub description: Option<LitStr>,
    pub tags: Option<Vec<LitStr>>,
    pub operation_id: Option<LitStr>,
    pub deprecated: bool,
    pub skip: Option<Vec<Ident>>,
    pub skip_all: bool,
}

impl OperationAttributes {
    pub fn merge_attributes(&mut self, attrs: &[syn::Attribute]) {
        let docstring = get_docstring(attrs).expect("Failed to parse docstring");
        if let Some(docstring) = docstring {
            self.description = Some(LitStr::new(&docstring, proc_macro2::Span::call_site()));
        }
    }
}

pub(crate) fn get_docstring(attrs: &[syn::Attribute]) -> syn::Result<Option<String>> {
    let string_literals = attrs
        .iter()
        .filter_map(|attr| match attr.meta {
            syn::Meta::NameValue(ref name_value) if name_value.path.is_ident("doc") => {
                Some(&name_value.value)
            }
            _ => None,
        })
        .map(|expr| match expr {
            syn::Expr::Lit(syn::ExprLit {
                lit: syn::Lit::Str(s),
                ..
            }) => Ok(s.value()),
            other => Err(syn::Error::new(
                other.span(),
                "Doc comment is not a string literal",
            )),
        })
        .collect::<Result<Vec<_>, _>>()?;

    if string_literals.is_empty() {
        return Ok(None);
    }

    let trimmed: Vec<_> = string_literals
        .iter()
        .flat_map(|lit| lit.split('\n').collect::<Vec<_>>())
        .map(|line| line.trim().to_string())
        .collect();

    Ok(Some(trimmed.join("\n")))
}
