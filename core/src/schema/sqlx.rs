use crate::{impl_oa_schema_passthrough};

#[cfg(feature = "json")]
impl_oa_schema_passthrough!(sqlx::types::Json<T>);