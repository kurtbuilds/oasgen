mod operation;
mod schema;

pub use operation::*;
pub use schema::*;

#[cfg(test)]
#[allow(non_snake_case)]
mod tests {
    use openapiv3::SchemaKind;
    use super::*;

    #[test]
    fn test_String_schema() {
        let s = String::schema().unwrap();
        let SchemaKind::Type(openapiv3::Type::String(_)) = s.schema_kind else { panic!() };
    }
}
