use crate::{OaSchema, Schema};

impl<T> OaSchema for sid::Sid<T> {
    fn schema() -> Schema {
        Schema::new_string()
    }
}