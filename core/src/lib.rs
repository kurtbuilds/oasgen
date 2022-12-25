mod operation;

pub use operation::*;

use openapiv3::{Schema, SchemaKind, SchemaData, ArrayType, Type, ReferenceOr};

pub trait OaSchema<Args = ()> {
    fn name() -> Option<&'static str>;
    fn schema_ref() -> Option<ReferenceOr<Schema>>;
    fn schema() -> Option<Schema>;
}

macro_rules! impl_oa_schema {
    ($t:ty,$schema:expr) => {
        impl OaSchema for $t {
            fn name() -> Option<&'static str> {
                None
            }

            fn schema_ref() -> Option<ReferenceOr<Schema>> {
                Some(ReferenceOr::Item($schema))
            }

            fn schema() -> Option<Schema> {
                Some($schema)
            }
        }
    }
}

impl OaSchema for () {
    fn name() -> Option<&'static str> {
        None
    }

    fn schema_ref() -> Option<ReferenceOr<Schema>> {
        None
    }

    fn schema() -> Option<Schema> {
        None
    }
}

impl_oa_schema!(usize, Schema::new_integer());
impl_oa_schema!(u32, Schema::new_integer());
impl_oa_schema!(i32, Schema::new_integer());
impl_oa_schema!(u64, Schema::new_integer());
impl_oa_schema!(i64, Schema::new_integer());
impl_oa_schema!(u16, Schema::new_integer());
impl_oa_schema!(i16, Schema::new_integer());
impl_oa_schema!(bool, Schema::new_bool());

impl_oa_schema!(String, Schema::new_string());

impl<T> OaSchema for Vec<T>
    where
        T: OaSchema,
{
    fn name() -> Option<&'static str> {
        None
    }

    fn schema_ref() -> Option<ReferenceOr<Schema>> {
        Some(ReferenceOr::Item(Schema {
            schema_data: SchemaData::default(),
            schema_kind: SchemaKind::Type(Type::Array(ArrayType {
                items: T::schema_ref().map(|r| r.boxed()),
                ..ArrayType::default()
            })),
        }))
    }

    fn schema() -> Option<Schema> {
        if let Some(schema) = T::schema() {
            Some(Schema::new_array(schema))
        } else {
            Some(Schema {
                schema_data: SchemaData::default(),
                schema_kind: SchemaKind::Type(Type::Array(ArrayType {
                    items: None,
                    ..ArrayType::default()
                })),
            })
        }
    }
}


impl<T> OaSchema for Option<T>
    where
        T: OaSchema,
{
    fn name() -> Option<&'static str> {
        T::name()
    }

    fn schema_ref() -> Option<ReferenceOr<Schema>> {
        T::schema_ref()
    }

    fn schema() -> Option<Schema> {
        T::schema().map(|mut schema| {
            schema.schema_data.nullable = true;
            schema
        })
    }
}

#[cfg(feature = "uuid")]
impl_oa_schema!(uuid::Uuid, Schema::new_string().with_format("uuid"));

impl<T> OaSchema for actix_web::web::Json<T>
    where
        T: OaSchema,
{
    fn name() -> Option<&'static str> {
        T::name()
    }

    fn schema_ref() -> Option<ReferenceOr<Schema>> {
        T::schema_ref()
    }

    fn schema() -> Option<Schema> {
        T::schema()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let s = String::schema().unwrap();
        let SchemaKind::Type(crate::Type::String(s)) = s.schema_kind else { panic!() };
    }
}
