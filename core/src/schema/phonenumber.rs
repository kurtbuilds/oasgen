use crate::{impl_oa_schema, Schema};

impl_oa_schema!(::phonenumber::PhoneNumber, Schema::new_string().with_format("e164-phone-number"));