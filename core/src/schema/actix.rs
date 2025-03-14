use openapiv3 as oa;
use openapiv3::{RefOr, Schema, SchemaKind, Type};

use crate::{OaParameter, OaSchema};

impl<T: OaSchema> OaParameter for actix_web::web::Json<T> {
    fn body_schema() -> Option<RefOr<Schema>> {
        T::body_schema()
    }
}

impl OaSchema for actix_web::HttpResponse {
    fn schema() -> Schema {
        Schema::new_any()
    }
}

impl<T> OaParameter for actix_web::web::Data<T> {}
impl OaParameter for actix_web::HttpRequest {}

impl<T: OaParameter> OaParameter for actix_web::web::Path<T> {
    fn parameters() -> Vec<RefOr<oa::Parameter>> {
        T::parameter_schemas()
            .into_iter()
            .flat_map(|s| s.into_item())
            .enumerate()
            .flat_map(|(i, s)| -> Box<dyn Iterator<Item = _>> {
                match s.kind {
                    SchemaKind::Type(Type::Object(o)) => Box::new(o.properties.into_iter()),
                    _ => Box::new(Some((format!("path{i}"), RefOr::Item(s))).into_iter()),
                }
            })
            .map(|(k, v)| {
                let mut param = oa::Parameter::path(k, v);
                param.required = true;
                RefOr::Item(param)
            })
            .collect()
    }
}

impl<T: OaParameter> OaParameter for actix_web::web::Query<T> {
    fn parameters() -> Vec<RefOr<oa::Parameter>> {
        T::parameter_schemas()
            .into_iter()
            .flat_map(|s| s.into_item())
            .flat_map(|s| match s.kind {
                SchemaKind::Type(Type::Object(o)) => Some(o.properties),
                _ => None,
            })
            .flatten()
            .map(|(k, v)| RefOr::Item(oa::Parameter::query(k, v)))
            .collect()
    }
}

#[cfg(feature = "qs")]
impl<T: OaParameter> OaParameter for serde_qs::actix::QsQuery<T> {
    fn parameters() -> Vec<RefOr<oa::Parameter>> {
        actix_web::web::Query::<T>::parameters()
    }
}
