use openapiv3::{Schema, SchemaKind, SchemaData, ArrayType, Type};

pub trait OaSchema<Args = ()> {
    fn schema_ref() -> Option<String>;
    fn schema() -> Option<Schema>;
}

macro_rules! impl_oa_schema {
    ($t:ty,$schema:expr) => {
        impl OaSchema for $t {
            fn schema_ref() -> Option<String> {
                None
            }

            fn schema() -> Option<Schema> {
                Some($schema)
            }
        }
    }
}

impl OaSchema for () {
    fn schema_ref() -> Option<String> {
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
    fn schema_ref() -> Option<String> {
        None
    }

    fn schema() -> Option<Schema> {
        if let Some(schema_ref) = T::schema_ref() {
            Some(Schema::new_array_ref(&schema_ref))
        } else if let Some(schema) = T::schema() {
            Some(Schema::new_array(schema))
        } else {
            Some(Schema{
                schema_data: SchemaData::default(),
                schema_kind: SchemaKind::Type(Type::Array(ArrayType {
                    items: None,
                    ..ArrayType::default()
                })),
            })
        }
    }
}

#[cfg(feature = "uuid")]
impl OaSchema for uuid::Uuid {
    fn schema_ref() -> Option<String> {
        None
    }

    fn schema() -> Option<Schema> {
        Some(Schema::new_string()
            .with_format("uuid"))
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
