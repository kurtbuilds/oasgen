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
        Some(vec![ReferenceOr::Item(oa::Parameter::Query {
            parameter_data: oa::ParameterData {
                name: "query".to_string(),
                description: None,
                required: false,
                deprecated: None,
                format: oa::ParameterSchemaOrContent::Schema(T::schema_ref().unwrap()),
                example: None,
                examples: Default::default(),
                explode: None,
                extensions: Default::default(),
            },
            allow_reserved: false,
            style: oa::QueryStyle::Form,
            allow_empty_value: None,
        })])
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
                        ReferenceOr::Item(oa::Parameter::Path {
                            parameter_data: oa::ParameterData {
                                name: stringify!($arg).to_string(),
                                description: None,
                                required: true,
                                deprecated: None,
                                format: oa::ParameterSchemaOrContent::Schema($arg::schema_ref().unwrap()),
                                example: None,
                                examples: Default::default(),
                                explode: None,
                                extensions: Default::default(),
                            },
                            style: oa::PathStyle::Simple,
                        })
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
