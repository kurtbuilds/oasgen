use openapiv3::{Schema, SchemaKind, SchemaData, ArrayType, Type, ReferenceOr};

#[cfg(feature = "actix")]
mod actix;

pub trait OaSchema<Args = ()> {
    fn schema_name() -> Option<&'static str>;
    fn schema_ref() -> Option<ReferenceOr<Schema>>;
    fn schema() -> Option<Schema>;
}

macro_rules! impl_oa_schema {
    ($t:ty,$schema:expr) => {
        impl OaSchema for $t {
            fn schema_name() -> Option<&'static str> {
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

#[macro_export]
macro_rules! impl_oa_schema_passthrough {
    ($t:ty) => {
        impl<T> OaSchema for $t where T: OaSchema {
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
    }
}

#[macro_export]
macro_rules! impl_oa_schema_none {
    ($t:ty) => {
        impl OaSchema for $t {
            fn schema_name() -> Option<&'static str> {
                None
            }

            fn schema_ref() -> Option<ReferenceOr<Schema>> {
                None
            }

            fn schema() -> Option<Schema> {
                None
            }
        }
    }
}

impl_oa_schema_none!(());

impl_oa_schema!(bool, Schema::new_bool());

impl_oa_schema!(usize, Schema::new_integer());
impl_oa_schema!(u32, Schema::new_integer());
impl_oa_schema!(i32, Schema::new_integer());
impl_oa_schema!(u64, Schema::new_integer());
impl_oa_schema!(i64, Schema::new_integer());
impl_oa_schema!(u16, Schema::new_integer());
impl_oa_schema!(i16, Schema::new_integer());

impl_oa_schema!(String, Schema::new_string());

impl<T> OaSchema for Vec<T>
    where
        T: OaSchema,
{
    fn schema_name() -> Option<&'static str> {
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
    fn schema_name() -> Option<&'static str> {
        T::schema_name()
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

impl<T, E> OaSchema for Result<T, E>
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

#[cfg(feature = "uuid")]
impl_oa_schema!(uuid::Uuid, Schema::new_string().with_format("uuid"));

#[cfg(feature = "time")]
impl_oa_schema!(::time::OffsetDateTime, Schema::new_string().with_format("date-time"));
#[cfg(feature = "time")]
impl_oa_schema!(::time::PrimitiveDateTime, Schema::new_string().with_format("date-time"));
#[cfg(feature = "time")]
impl_oa_schema!(::time::Date, Schema::new_string().with_format("date"));
#[cfg(feature = "time")]
impl_oa_schema!(::time::Time, Schema::new_string().with_format("time"));

#[cfg(feature = "chrono")]
impl_oa_schema!(::chrono::DateTime<::chrono::Utc>, Schema::new_string().with_format("date-time"));
#[cfg(feature = "chrono")]
impl_oa_schema!(::chrono::NaiveDateTime, Schema::new_string().with_format("date-time"));

impl_oa_schema!(serde_json::Value, Schema::new_object());