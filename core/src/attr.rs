use structmeta::StructMeta;
use syn::LitStr;

/// Available attributes on a struct
#[derive(StructMeta, Default)]
pub struct OpenApiAttributes {
    pub skip: bool,
    pub skip_serializing_if: Option<LitStr>,
}

impl TryFrom<&Vec<syn::Attribute>> for OpenApiAttributes {
    type Error = syn::Error;

    fn try_from(attrs: &Vec<syn::Attribute>) -> Result<Self, Self::Error> {
        let mut attrs = attrs.into_iter().filter_map(|attr| {
            attr.path().get_ident().and_then(|ident| {
                if ident == "openapi" || ident == "serde" {
                    let attrs: OpenApiAttributes = attr.parse_args().ok()?;
                    Some(attrs)
                } else {
                    None
                }
            })
        });
        let mut result = attrs.next().unwrap_or_default();
        for attr in attrs {
            if attr.skip {
                result.skip = true;
            }
            if attr.skip_serializing_if.is_some() {
                result.skip_serializing_if = attr.skip_serializing_if;
            }
        }
        Ok(result)
    }
}
