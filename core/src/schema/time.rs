use crate::{impl_oa_schema, Schema};

impl_oa_schema!(::time::OffsetDateTime, Schema::new_string().with_format("date-time"));
impl_oa_schema!(::time::PrimitiveDateTime, Schema::new_string().with_format("date-time"));
impl_oa_schema!(::time::Date, Schema::new_string().with_format("date"));
impl_oa_schema!(::time::Time, Schema::new_string().with_format("time"));
