use openapiv3 as oa;
use openapiv3::ReferenceOr;
use crate::{impl_oa_schema_none, OaSchema};

impl<T: OaSchema> OaSchema for actix_web::web::Json<T> {
    fn body_schema() -> Option<ReferenceOr<oa::Schema>> {
        T::schema_ref()
    }
}

impl<T> OaSchema for actix_web::web::Data<T> {}

impl_oa_schema_none!(actix_web::HttpRequest);
impl_oa_schema_none!(actix_web::HttpResponse);

macro_rules! construct_path {
    ($($arg:ident),+) => {
        impl< $($arg),+ > OaSchema for actix_web::web::Path<( $($arg),+,)>
            where
                $($arg: OaSchema),+
        {
            fn parameters() -> Option<Vec<ReferenceOr<oa::Parameter>>> {
                Some(vec![
                    $(
                        ReferenceOr::Item(oa::Parameter::Path {
                            parameter_data: oa::ParameterData {
                                name: stringify!($arg).to_string(),
                                description: None,
                                required: true,
                                deprecated: None,
                                format: oa::ParameterSchemaOrContent::Schema($arg::schema_ref().unwrap()),
                                example: None,
                                examples: Default::default(),
                                explode: None,
                                extensions: Default::default(),
                            },
                            style: oa::PathStyle::Simple,
                        })
                    ),+
                ])
            }
        }
    };
}

construct_path!(A1);
construct_path!(A1, A2);
construct_path!(A1, A2, A3);

impl<T: OaSchema> OaSchema for actix_web::web::Query<T> {
    fn parameters() -> Option<Vec<ReferenceOr<oa::Parameter>>> {
        let p = oa::Parameter::query("query", T::schema_ref().unwrap());
        Some(vec![ReferenceOr::Item(p)])
    }
}

#[cfg(feature = "qs")]
impl<T: OaSchema> OaSchema for serde_qs::actix::QsQuery<T> {
    fn parameters() -> Option<Vec<ReferenceOr<oa::Parameter>>> {
        let p = oa::Parameter::query("query", T::schema_ref().unwrap());
        Some(vec![ReferenceOr::Item(p)])
    }
}