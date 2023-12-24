use std::collections::HashMap;

use openapiv3 as oa;
use openapiv3::{ReferenceOr, Schema};

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
    fn schema() -> Schema;
    fn schema_ref() -> ReferenceOr<Schema> {
        ReferenceOr::Item(Self::schema())
    }

    fn parameters() -> Vec<ReferenceOr<oa::Parameter>> {
        Vec::new()
    }

    fn body_schema() -> Option<ReferenceOr<Schema>> {
        Some(Self::schema_ref())
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
            fn schema() -> $crate::Schema {
                $schema
            }
        }
    };
}

#[macro_export]
macro_rules! impl_oa_schema_passthrough {
    ($t:ty) => {
        impl<T> $crate::OaSchema for $t where T: $crate::OaSchema {
            fn schema_ref() -> $crate::ReferenceOr<$crate::Schema> {
                T::schema_ref()
            }

            fn schema() -> $crate::Schema {
                T::schema()
            }
        }
    };
}

impl OaSchema for () {
    fn schema() -> Schema {
        panic!("Call body_schema() for (), not schema().")
    }

    fn body_schema() -> Option<ReferenceOr<Schema>> { None }
}

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
    fn schema_ref() -> ReferenceOr<Schema> {
        let inner = T::schema_ref();
        ReferenceOr::Item(Schema::new_array(inner))
    }

    fn schema() -> Schema {
        let inner = T::schema();
        Schema::new_array(inner)
    }
}

impl<T> OaSchema for Option<T> where T: OaSchema {
    fn schema_ref() -> ReferenceOr<Schema> {
        let mut schema = T::schema_ref();
        let Some(s) = schema.as_mut() else {
            return schema;
        };
        s.nullable = true;
        schema
    }

    fn schema() -> Schema {
        let mut schema = T::schema();
        schema.nullable = true;
        schema
    }
}

impl<T, E> OaSchema for Result<T, E>
    where
        T: OaSchema,
{
    fn schema_ref() -> ReferenceOr<Schema> {
        T::schema_ref()
    }

    fn schema() -> Schema {
        T::schema()
    }
}

impl<K, V> OaSchema for HashMap<K, V>
    where
        V: OaSchema,
{
    fn schema_ref() -> ReferenceOr<Schema> {
        ReferenceOr::Item(Schema::new_map(V::schema_ref()))
    }

    fn schema() -> Schema {
        Schema::new_map(V::schema())
    }
}

#[cfg(feature = "uuid")]
impl_oa_schema!(uuid::Uuid, Schema::new_string().with_format("uuid"));

impl_oa_schema!(serde_json::Value, Schema::new_object());
