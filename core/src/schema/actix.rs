use openapiv3 as oa;
use openapiv3::{ReferenceOr, Schema};

use crate::OaSchema;

impl<T: OaSchema> OaSchema for actix_web::web::Json<T> {
    fn schema() -> Schema {
        panic!("Call body_schema() for Json, not schema().")
    }

    fn body_schema() -> Option<ReferenceOr<Schema>> {
        T::body_schema()
    }
}

impl<T> OaSchema for actix_web::web::Data<T> {
    fn schema() -> Schema {
        panic!("Call parameters() for Data, not schema().");
    }
}

impl OaSchema for actix_web::HttpRequest {
    fn schema() -> Schema {
        panic!("Call parameters() for HttpRequest, not schema().");
    }
}

impl OaSchema for actix_web::HttpResponse {
    fn schema() -> Schema {
        panic!("Call body_schema() for HttpResponse, not schema().");
    }
}

impl<T: OaSchema> OaSchema for actix_web::web::Path<T> {
    fn schema() -> Schema {
        panic!("Call parameters() for Path, not schema().");
    }

    fn parameters() -> Vec<ReferenceOr<oa::Parameter>> {
        T::parameters()
    }

    fn body_schema() -> Option<ReferenceOr<Schema>> {
        None
    }
}

impl<T: OaSchema> OaSchema for actix_web::web::Query<T> {
    fn schema() -> Schema {
        panic!("Call parameters() for Query, not schema().");
    }

    fn parameters() -> Vec<ReferenceOr<oa::Parameter>> {
        T::parameters()
    }
    fn body_schema() -> Option<ReferenceOr<Schema>> {
        None
    }
}

#[cfg(feature = "qs")]
impl<T: OaSchema> OaSchema for serde_qs::actix::QsQuery<T> {
    fn schema() -> Schema {
        panic!("Call parameters() for QsQuery, not schema().");
    }

    fn parameters() -> Vec<ReferenceOr<oa::Parameter>> {
        let p = oa::Parameter::query("query", T::schema_ref());
        vec![ReferenceOr::Item(p)]
    }
}