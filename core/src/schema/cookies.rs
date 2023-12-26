use openapiv3::{ReferenceOr, Schema};

impl crate::OaSchema for tower_cookies::Cookies {
    fn schema() -> Schema {
        Schema::new_object()
    }

    fn body_schema() -> Option<ReferenceOr<Schema>> {
        None
    }
}