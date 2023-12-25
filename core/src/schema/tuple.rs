use crate::OaSchema;
use openapiv3::{RefOr, Schema, Parameter};

impl<A1> OaSchema for (A1,) where A1: OaSchema {
    fn schema() -> Schema {
        panic!("Call parameters() or body_schema() for tuples, not schema()")
    }

    fn parameters() -> Vec<RefOr<Parameter>> {
        vec![Parameter::path("param1", A1::schema_ref()).into()]
    }

    fn body_schema() -> Option<RefOr<Schema>> {
        A1::body_schema()
    }
}

impl<A1, A2> OaSchema for (A1, A2)
    where
        A1: OaSchema,
        A2: OaSchema,
{
    fn schema() -> Schema {
        panic!("Call parameters() or body_schema() for tuples, not schema()")
    }

    fn parameters() -> Vec<RefOr<Parameter>> {
        vec![
            Parameter::path("param1", A1::schema_ref()).into(),
            Parameter::path("param2", A2::schema_ref()).into(),
        ]
    }

    fn body_schema() -> Option<RefOr<Schema>> {
        A2::body_schema()
    }
}

impl<A1, A2, A3> OaSchema for (A1, A2, A3)
    where
        A1: OaSchema,
        A2: OaSchema,
        A3: OaSchema,
{
    fn schema() -> Schema {
        panic!("Call parameters() or body_schema() for tuples, not schema()")
    }

    fn parameters() -> Vec<RefOr<Parameter>> {
        vec![
            Parameter::path("param1", A1::schema_ref()).into(),
            Parameter::path("param2", A2::schema_ref()).into(),
            Parameter::path("param3", A3::schema_ref()).into(),
        ]
    }

    fn body_schema() -> Option<RefOr<Schema>> {
        A3::body_schema()
    }
}