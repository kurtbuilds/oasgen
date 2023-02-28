use crate::{impl_oa_schema_passthrough, OaSchema};

impl_oa_schema_passthrough!(axum::Json<T>);

impl<T> OaSchema for axum::extract::Extension<T> {}

impl<T> OaSchema for axum::extract::State<T> {}