use openapiv3::{Operation, Parameter, RefOr, Schema};

pub struct OperationRegister {
    pub name: &'static str,
    pub constructor: &'static (dyn Sync + Send + Fn() -> Operation),
}

pub trait OaParameter {
    fn body_schema() -> Option<RefOr<Schema>> {
        None
    }
    fn parameter_schemas() -> Vec<RefOr<Schema>> {
        Vec::new()
    }
    fn parameters() -> Vec<RefOr<Parameter>> {
        Vec::new()
    }
}

impl<T, E> OaParameter for Result<T, E>
where
    T: OaParameter,
{
    fn body_schema() -> Option<RefOr<Schema>> {
        T::body_schema()
    }
}

inventory::collect!(OperationRegister);
