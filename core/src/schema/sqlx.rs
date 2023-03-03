use crate::{impl_oa_schema_passthrough};

#[cfg(feature = "postgres")]
impl_oa_schema_passthrough!(sqlx::types::Json<T>);