use std::collections::HashMap;

use oa::AdditionalProperties;
use openapiv3 as oa;
use openapiv3::{ObjectType, ReferenceOr, Schema, SchemaData, SchemaKind, Type};

#[cfg(feature = "actix")]
mod actix;

#[cfg(feature = "axum")]
mod axum;

#[cfg(feature = "chrono")]
mod chrono;
#[cfg(feature = "cookies")]
mod cookies;
#[cfg(feature = "phonenumber")]
mod phonenumber;
#[cfg(feature = "sqlx")]
mod sqlx;
#[cfg(feature = "time")]
mod time;

mod http;
#[cfg(feature = "sid")]
mod sid;

pub trait OaSchema {
    fn schema_ref() -> Option<ReferenceOr<Schema>> {
        None
    }

    fn schema() -> Option<Schema> {
        None
    }

    fn parameters() -> Option<Vec<ReferenceOr<oa::Parameter>>> {
        None
    }

    fn body_schema() -> Option<ReferenceOr<Schema>> {
        None
    }
}

pub struct SchemaRegister {
    pub name: &'static str,
    pub constructor: &'static (dyn Sync + Send + Fn() -> Schema),
}

inventory::collect!(SchemaRegister);

#[macro_export]
macro_rules! impl_oa_schema {
    ($t:ty,$schema:expr) => {
        impl $crate::OaSchema for $t {
            fn schema_ref() -> Option<$crate::ReferenceOr<$crate::Schema>> {
                Some($crate::ReferenceOr::Item($schema))
            }

            fn schema() -> Option<$crate::Schema> {
                Some($schema)
            }
        }
    };
}

#[macro_export]
macro_rules! impl_oa_schema_passthrough {
    ($t:ty) => {
        impl<T> $crate::OaSchema for $t where T: $crate::OaSchema {
            fn schema_ref() -> Option<$crate::ReferenceOr<$crate::Schema>> {
                T::schema_ref()
            }

            fn schema() -> Option<$crate::Schema> {
                T::schema()
            }
        }
    };
}

#[macro_export]
macro_rules! impl_oa_schema_none {
    ($t:ty) => {
        impl $crate::OaSchema for $t {}
    };
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
impl_oa_schema!(f32, Schema::new_number());
impl_oa_schema!(f64, Schema::new_number());

impl_oa_schema!(String, Schema::new_string());

impl<T> OaSchema for Vec<T> where T: OaSchema {
    fn schema_ref() -> Option<ReferenceOr<Schema>> {
        let schema = if let Some(schema) = T::schema_ref() {
            Schema::new_array(schema)
        } else {
            Schema::new_any_array()
        };
        Some(ReferenceOr::Item(schema))
    }

    fn schema() -> Option<Schema> {
        let schema = if let Some(schema) = T::schema() {
            Schema::new_array(ReferenceOr::Item(schema))
        } else {
            Schema::new_any_array()
        };
        Some(schema)
    }
}

impl<T> OaSchema for Option<T> where T: OaSchema {
    fn schema_ref() -> Option<ReferenceOr<Schema>> {
        let mut schema = T::schema_ref();
        let Some(s) = &mut schema else {
            return schema
        };
        let Some(s) = s.as_mut() else {
            return schema
        };
        s.schema_data.nullable = true;
        schema
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
    fn schema_ref() -> Option<ReferenceOr<Schema>> {
        T::schema_ref()
    }

    fn schema() -> Option<Schema> {
        T::schema()
    }
}

impl<K, V> OaSchema for HashMap<K, V>
where
    V: OaSchema,
{
    fn schema_ref() -> Option<ReferenceOr<Schema>> {
        Some(ReferenceOr::Item(Schema {
            schema_data: SchemaData::default(),
            schema_kind: SchemaKind::Type(Type::Object(ObjectType {
                additional_properties: Some(AdditionalProperties::Schema(Box::new(
                    V::schema_ref()?
                ))),
                ..ObjectType::default()
            })),
        }))
    }

    fn schema() -> Option<Schema> {
        Some(Schema {
            schema_data: SchemaData::default(),
            schema_kind: SchemaKind::Type(Type::Object(ObjectType {
                additional_properties: V::schema()
                    .map(|s| AdditionalProperties::Schema(Box::new(ReferenceOr::Item(s)))),
                ..ObjectType::default()
            })),
        })
    }
}

#[cfg(feature = "uuid")]
impl_oa_schema!(uuid::Uuid, Schema::new_string().with_format("uuid"));

impl_oa_schema!(serde_json::Value, Schema::new_object());
