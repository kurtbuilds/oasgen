use openapiv3::{ReferenceOr, Schema};

use crate::OaSchema;

impl<T: OaSchema> OaSchema for axum::extract::Json<T> {
    fn body_schema() -> Option<ReferenceOr<Schema>> {
        T::schema_ref()
    }
}

impl<T> OaSchema for axum::extract::Extension<T> {}

impl<T> OaSchema for axum::extract::State<T> {}

impl<T> OaSchema for axum::http::Response<T> {}

impl<T> OaSchema for axum::http::Request<T> {}

impl<T> OaSchema for axum::extract::ConnectInfo<T> {}

impl OaSchema for axum::http::HeaderMap {}

// TODO fill this out
impl<T> OaSchema for axum::extract::Query<T> {}

// TODO fill this out
impl<T> OaSchema for axum::extract::Path<T> {}

impl OaSchema for axum::http::request::Parts {}
