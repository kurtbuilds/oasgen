use openapiv3::{Schema, ReferenceOr};
use crate::{impl_oa_schema_none, impl_oa_schema_passthrough, OaSchema};

impl_oa_schema_passthrough!(actix_web::web::Json<T>);
impl_oa_schema_passthrough!(sqlx::types::Json<T>);

impl_oa_schema_none!(actix_web::web::Data<sqlx::postgres::PgPool>);
impl_oa_schema_none!(actix_web::HttpRequest);
impl_oa_schema_none!(actix_web::HttpResponse);