use crate::impl_oa_schema;

impl_oa_schema!(::chrono::NaiveDate, crate::Schema::new_string().with_format("date"));
impl_oa_schema!(::chrono::DateTime<::chrono::Utc>, crate::Schema::new_string().with_format("date-time"));
impl_oa_schema!(::chrono::DateTime<::chrono::FixedOffset>, crate::Schema::new_string().with_format("date-time"));
impl_oa_schema!(::chrono::NaiveDateTime, crate::Schema::new_string().with_format("date-time"));