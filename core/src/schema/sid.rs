use crate::{OaSchema, Schema};

impl<T> OaSchema for sid::Sid<T> {
    fn schema() -> Option<Schema> {
        Some(Schema::new_string())
    }
}