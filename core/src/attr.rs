use structmeta::StructMeta;

/// Available attributes on a struct
#[derive(StructMeta, Debug, Default)]
pub struct OpenApiAttributes {
    pub skip: bool,
}

impl TryFrom<&Vec<syn::Attribute>> for OpenApiAttributes {
    type Error = syn::Error;

    fn try_from(attrs: &Vec<syn::Attribute>) -> Result<Self, Self::Error> {
        let mut attrs = attrs.into_iter().filter_map(|attr| {
            attr.path.get_ident().and_then(|ident| {
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
        }
        Ok(result)
    }
}
