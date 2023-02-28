use openapiv3 as oa;
use openapiv3::{ReferenceOr};
use crate::{impl_oa_schema_none, impl_oa_schema_passthrough, OaSchema};

impl_oa_schema_passthrough!(actix_web::web::Json<T>);

impl<T> OaSchema for actix_web::web::Data<T> {

}

impl_oa_schema_none!(actix_web::HttpRequest);
impl_oa_schema_none!(actix_web::HttpResponse);

macro_rules! construct_path {
    ($($arg:ident),+) => {
        impl< $($arg),+ > OaSchema for actix_web::web::Path<( $($arg),+,)>
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
                                format: oa::ParameterSchemaOrContent::Schema(ReferenceOr::Item(
                                    $arg::schema().unwrap(),
                                )),
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

impl<T: OaSchema> OaSchema for actix_web::web::Query<T> {
    fn parameters() -> Option<Vec<ReferenceOr<oa::Parameter>>> {
        Some(vec![ReferenceOr::Item(oa::Parameter::Query {
            parameter_data: oa::ParameterData {
                name: "query".to_string(),
                description: None,
                required: false,
                deprecated: None,
                format: oa::ParameterSchemaOrContent::Schema(ReferenceOr::Item(
                    T::schema().unwrap(),
                )),
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