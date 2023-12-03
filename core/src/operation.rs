use openapiv3::Operation;

pub struct OperationRegister {
    // module_path: &'static str,
    pub name: &'static str,
    pub constructor: &'static (dyn Sync + Send + Fn() -> Operation),
}

inventory::collect!(OperationRegister);