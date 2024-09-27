use openapiv3::{RefOr, Schema};
use crate::{OaParameter, OaSchema};

impl<A: OaSchema> OaParameter for A {
    fn parameter_schemas() -> Vec<RefOr<Schema>> {
        vec![RefOr::Item(A::schema())]
    }
    fn body_schema() -> Option<RefOr<Schema>> {
        A::body_schema()
    }
}

impl<A1: OaSchema> OaParameter for (A1,) {
    fn parameter_schemas() -> Vec<RefOr<Schema>> {
        vec![A1::schema_ref()]
    }
}

impl<A1: OaSchema, A2: OaSchema> OaParameter for (A1, A2) {
    fn parameter_schemas() -> Vec<RefOr<Schema>> {
        vec![
            A1::schema_ref(),
            A2::schema_ref(),
        ]
    }
}

impl<A1: OaSchema, A2: OaSchema, A3: OaSchema> OaParameter for (A1, A2, A3) {
    fn parameter_schemas() -> Vec<RefOr<Schema>> {
        vec![
            A1::schema_ref(),
            A2::schema_ref(),
            A3::schema_ref(),
        ]
    }
}