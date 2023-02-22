use openapiv3 as oa;
use openapiv3::{Schema, ReferenceOr};
use crate::{impl_oa_schema_none, impl_oa_schema_passthrough, OaSchema};

impl_oa_schema_passthrough!(actix_web::web::Json<T>);
impl_oa_schema_passthrough!(sqlx::types::Json<T>);

impl_oa_schema_none!(actix_web::web::Data<sqlx::postgres::PgPool>);
impl_oa_schema_none!(actix_web::HttpRequest);
impl_oa_schema_none!(actix_web::HttpResponse);

impl<T> OaSchema for actix_web::web::Path<T>
    where
        T: OaSchema,
{
    fn schema_name() -> Option<&'static str> {
        None
    }

    fn schema_ref() -> Option<ReferenceOr<Schema>> {
        None
    }

    fn schema() -> Option<Schema> {
        None
    }

    fn parameter() -> Option<ReferenceOr<oa::Parameter>> {
        Some(ReferenceOr::Item(oa::Parameter::Path {
            parameter_data: oa::ParameterData {
                name: "placeholder".to_string(),
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
            style: oa::PathStyle::Simple,
        }))
    }
}

impl<T: OaSchema> OaSchema for actix_web::web::Query<T> {
    fn schema_name() -> Option<&'static str> {
        None
    }

    fn schema_ref() -> Option<ReferenceOr<Schema>> {
        None
    }

    fn schema() -> Option<Schema> {
        None
    }

    fn parameter() -> Option<ReferenceOr<oa::Parameter>> {
        Some(ReferenceOr::Item(oa::Parameter::Query {
            parameter_data: oa::ParameterData {
                name: "placeholder".to_string(),
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
        }))
    }
}
