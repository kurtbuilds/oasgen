use openapiv3::{Schema, ReferenceOr};
use crate::OaSchema;


impl<T> OaSchema for actix_web::web::Json<T>
    where
        T: OaSchema,
{
    fn schema_name() -> Option<&'static str> {
        T::schema_name()
    }

    fn schema_ref() -> Option<ReferenceOr<Schema>> {
        T::schema_ref()
    }

    fn schema() -> Option<Schema> {
        T::schema()
    }
}