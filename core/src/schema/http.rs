use http::{Method, Version, Uri};
use openapiv3::Schema;
use crate::OaSchema;

impl OaSchema for Method {
    fn schema() -> Schema {
        Schema::new_string()
    }
}

impl OaSchema for Version {
    fn schema() -> Schema {
        Schema::new_string()
    }
}

impl OaSchema for Uri {
    fn schema() -> Schema {
        Schema::new_string()
    }
}