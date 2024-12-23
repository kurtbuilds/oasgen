use openapiv3::{self as oa};
use openapiv3::{RefOr, Schema, SchemaKind, Type};

use crate::{OaParameter, OaSchema};

impl<T: OaParameter> OaParameter for actix_web::web::Json<T> {
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
            .map(|s| RefOr::Item(oa::Parameter::path("path", s)))
            .collect()
    }
}

impl<T: OaParameter> OaParameter for actix_web::web::Query<T> {
    fn parameters() -> Vec<RefOr<oa::Parameter>> {
        T::parameter_schemas()
            .into_iter()
            .flat_map(|s| s.into_item())
            .flat_map(|s| match s.kind {
                SchemaKind::Type(Type::Object(o)) => { Some(o.properties) }
                _ => None
            })
            .flatten()
            .map(|(k, v)| RefOr::Item(oa::Parameter::query(k, v)))
            .collect()
    }
}

impl OaParameter for actix_web::web::Bytes {
    fn body_schema() -> Option<RefOr<Schema>> {
        Some(RefOr::Item(Schema {
            data: oa::SchemaData {
                title: Some("Binary".to_string()),
                description: Some("Binary content".to_string()),
                ..Default::default()
            },
            kind: oa::SchemaKind::Type(oa::Type::String(oa::StringType {
                format: oa::VariantOrUnknownOrEmpty::Item(oa::StringFormat::Binary),
                ..Default::default()
            }))
        }))
    }
}

impl OaParameter for actix_files::NamedFile {
    fn body_schema() -> Option<RefOr<Schema>> {
        Some(RefOr::Item(Schema {
            data: oa::SchemaData {
                title: Some("File".to_string()),
                description: Some("Regular file".to_string()),
                ..Default::default()
            },
            kind: oa::SchemaKind::Type(oa::Type::String(oa::StringType {
                format: oa::VariantOrUnknownOrEmpty::Item(oa::StringFormat::Binary),
                ..Default::default()
            }))
        }))
    }
}

impl<L, R> OaParameter for actix_web::Either<L, R>
where
    L: OaParameter,
    R: OaParameter,
{
    fn body_schema() -> Option<RefOr<Schema>> {
        L::body_schema().or_else(|| R::body_schema())
    }
}

#[cfg(feature = "qs")]
impl<T: OaParameter> OaParameter for serde_qs::actix::QsQuery<T> {
    fn parameters() -> Vec<RefOr<oa::Parameter>> {
        T::parameter_schemas()
            .into_iter()
            .map(|s| RefOr::Item(oa::Parameter::query("query", s)))
            .collect()
    }
}
