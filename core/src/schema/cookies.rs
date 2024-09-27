use openapiv3::{Schema};

impl crate::OaSchema for tower_cookies::Cookies {
    fn schema() -> Schema {
        Schema::new_object()
    }
}