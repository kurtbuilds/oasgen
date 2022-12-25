use std::future::Future;
use openapiv3::{MediaType, Operation, ReferenceOr, RequestBody, Response, Responses, Schema, StatusCode};
use crate::OaSchema;

pub trait OaOperation<Args, Fut, Output> {
    fn operation() -> Operation;
}


fn type_name_to_operation_id(type_name: &str) -> Option<String> {
    Some(type_name.split("::").skip(1).collect::<Vec<_>>().join("_"))
}

// impl<F, A, Fut, Output> OaOperation<(A, Fut, Output)> for F
//     where
//         F: Fn(A) -> Fut,
//         Fut: Future<Output=Output>,
impl<F, A, Fut, Output> OaOperation<(A, ), Fut, Output> for F
    where
        F: Fn(A) -> Fut,
        A: OaSchema,
        Fut: Future<Output=Output>,
        Output: OaSchema,
{
    // type Args = Args;
    fn operation() -> Operation {
        let s = A::schema_ref().unwrap();
        let mut content = indexmap::IndexMap::new();
        content.insert("application/json".to_string(), MediaType {
            schema: Some(s),
            ..MediaType::default()
        });
        let body = RequestBody {
            content,
            required: true,
            ..RequestBody::default()
        };
        let mut responses = Responses::default();
        responses.responses.insert(StatusCode::Code(200), ReferenceOr::Item({
            let mut content = indexmap::IndexMap::new();
            content.insert("application/json".to_string(), MediaType {
                schema: Some(Output::schema_ref().unwrap()),
                ..MediaType::default()
            });
            Response {
                content,
                ..Response::default()
            }
        }));
        Operation {
            operation_id: type_name_to_operation_id(std::any::type_name::<F>()),
            request_body: Some(ReferenceOr::Item(body)),
            responses,
            ..Operation::default()
        }
    }
}