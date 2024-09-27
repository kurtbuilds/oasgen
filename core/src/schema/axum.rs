use openapiv3::{RefOr, Schema, SchemaKind, Type};
use openapiv3 as oa;

use crate::{OaParameter, OaSchema};

impl<T> OaSchema for http::Response<T> {
    fn schema() -> Schema {
        Schema::new_any()
    }
}

impl<T: OaSchema> OaParameter for axum::extract::Json<T> {
    fn body_schema() -> Option<RefOr<Schema>> {
        T::body_schema()
    }
}
impl<T> OaParameter for axum::extract::Extension<T> {}
impl<T> OaParameter for axum::extract::State<T> {}
impl<T> OaParameter for http::Request<T> {}
impl<T> OaParameter for axum::extract::ConnectInfo<T> {}
impl OaParameter for http::HeaderMap {}
impl OaParameter for http::request::Parts {}

impl<T: OaParameter> OaParameter for axum::extract::Query<T> {
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

impl<T: OaParameter> OaParameter for axum::extract::Path<T> {
    fn parameters() -> Vec<RefOr<oa::Parameter>> {
        T::parameter_schemas()
            .into_iter()
            .map(|s| RefOr::Item(oa::Parameter::path("path", s)))
            .collect()
    }
}

#[cfg(feature = "qs")]
impl<T: OaParameter> OaParameter for serde_qs::axum::QsQuery<T> {
    fn parameters() -> Vec<RefOr<oa::Parameter>> {
        T::parameter_schemas()
            .into_iter()
            .map(|s| RefOr::Item(oa::Parameter::query("query", s)))
            .collect()
    }
}