use openapiv3::{ReferenceOr, Schema};
use openapiv3 as oa;

use crate::OaSchema;

impl<T: OaSchema> OaSchema for axum::extract::Json<T> {
    fn body_schema() -> Option<ReferenceOr<Schema>> {
        T::schema_ref()
    }
}

impl<T> OaSchema for axum::extract::Extension<T> {}

impl<T> OaSchema for axum::extract::State<T> {}

impl<T> OaSchema for axum::http::Response<T> {}

impl<T> OaSchema for axum::http::Request<T> {}

impl<T> OaSchema for axum::extract::ConnectInfo<T> {}

impl OaSchema for axum::http::HeaderMap {}

impl<T: OaSchema> OaSchema for axum::extract::Query<T> {
    fn parameters() -> Option<Vec<ReferenceOr<oa::Parameter>>> {
        let p = oa::Parameter::query("query", T::schema_ref().unwrap());
        Some(vec![ReferenceOr::Item(p)])
    }
}

macro_rules! construct_path {
    ($($arg:ident),+) => {
        impl< $($arg),+ > OaSchema for axum::extract::Path<( $($arg),+,)>
            where
                $($arg: OaSchema),+
        {
            fn parameters() -> Option<Vec<ReferenceOr<oa::Parameter>>> {
                Some(vec![
                    $(
                        ReferenceOr::Item(oa::Parameter::path(stringify!($arg), $arg::schema_ref().unwrap()))
                    ),+
                ])
            }
        }
    };
}

construct_path!(A1);
construct_path!(A1, A2);
construct_path!(A1, A2, A3);

impl OaSchema for axum::http::request::Parts {}

#[cfg(feature = "qs")]
impl<T: OaSchema> OaSchema for serde_qs::axum::QsQuery<T> {
    fn parameters() -> Option<Vec<ReferenceOr<oa::Parameter>>> {
        let p = oa::Parameter::query("query", T::schema_ref().unwrap());
        Some(vec![ReferenceOr::Item(p)])
    }
}