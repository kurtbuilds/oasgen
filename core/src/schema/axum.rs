use openapiv3::{ReferenceOr, Schema};
use openapiv3 as oa;

use crate::OaSchema;

impl<T: OaSchema> OaSchema for axum::extract::Json<T> {
    fn schema() -> Schema {
        panic!("Call body_schema() for Json, not schema().")
    }

    fn body_schema() -> Option<ReferenceOr<Schema>> {
        T::body_schema()
    }
}

impl<T> OaSchema for axum::extract::Extension<T> {
    fn schema() -> Schema {
        panic!("Call parameters() for Extension, not schema().")
    }
    fn body_schema() -> Option<ReferenceOr<Schema>> { None }
}

impl<T> OaSchema for axum::extract::State<T> {
    fn schema() -> Schema {
        panic!("Call parameters() for State, not schema().")
    }
}

impl<T> OaSchema for http::Response<T> {
    fn schema() -> Schema {
        Schema::new_any()
    }
}

impl<T> OaSchema for http::Request<T> {
    fn schema() -> Schema {
        panic!("Call parameters() for Request, not schema().")
    }
    fn body_schema() -> Option<ReferenceOr<Schema>> { None }
}

impl<T> OaSchema for axum::extract::ConnectInfo<T> {
    fn body_schema() -> Option<ReferenceOr<Schema>> { None }
    fn schema() -> Schema {
        panic!("Call parameters() for ConnectInfo, not schema().")
    }
}

impl OaSchema for http::HeaderMap {
    fn schema() -> Schema {
        panic!("Call parameters() for HeaderMap, not schema().")
    }
    fn body_schema() -> Option<ReferenceOr<Schema>> { None }
}

impl<T: OaSchema> OaSchema for axum::extract::Query<T> {
    fn schema() -> Schema {
        panic!("Call parameters() for Query, not schema().")
    }

    fn parameters() -> Vec<ReferenceOr<oa::Parameter>> {
        let p = oa::Parameter::query("query", T::schema_ref());
        vec![ReferenceOr::Item(p)]
    }

    fn body_schema() -> Option<ReferenceOr<Schema>> { None }
}

macro_rules! construct_path {
    ($($arg:ident),+) => {
        impl< $($arg),+ > OaSchema for axum::extract::Path<( $($arg),+,)>
            where
                $($arg: OaSchema),+
        {
            fn schema() -> Schema {
                panic!("Call parameters() for Path, not schema().")
            }

            fn parameters() -> Vec<ReferenceOr<oa::Parameter>> {
                vec![
                    $(
                        ReferenceOr::Item(oa::Parameter::path(stringify!($arg), $arg::schema_ref()))
                    ),+
                ]
            }
        }
    };
}

construct_path!(A1);
construct_path!(A1, A2);
construct_path!(A1, A2, A3);

impl OaSchema for http::request::Parts {
    fn schema() -> Schema {
        panic!("Call parameters() for Parts, not schema().")
    }
    fn body_schema() -> Option<ReferenceOr<Schema>> { None }
}

#[cfg(feature = "qs")]
impl<T: OaSchema> OaSchema for serde_qs::axum::QsQuery<T> {
    fn schema() -> Schema {
        panic!("Call parameters() for QsQuery, not schema().")
    }

    fn parameters() -> Vec<ReferenceOr<oa::Parameter>> {
        let p = oa::Parameter::query("query", T::schema_ref());
        vec![ReferenceOr::Item(p)]
    }
    fn body_schema() -> Option<ReferenceOr<Schema>> { None }
}
