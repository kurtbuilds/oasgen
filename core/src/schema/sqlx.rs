use crate::{impl_oa_schema_passthrough};

impl_oa_schema_passthrough!(sqlx::types::Json<T>);