use crate::{impl_oa_schema_passthrough, OaSchema};

impl_oa_schema_passthrough!(axum::Json<T>);

impl<T> OaSchema for axum::extract::Extension<T> {}

impl<T> OaSchema for axum::extract::State<T> {}

impl<T> OaSchema for axum::http::Response<T> {}

impl<T> OaSchema for axum::http::Request<T> {}

impl<T> OaSchema for axum::extract::ConnectInfo<T> {}

// TODO fill this out
impl<T> OaSchema for axum::extract::Query<T> {}

// TODO fill this out
impl<T> OaSchema for axum::extract::Path<T> {}