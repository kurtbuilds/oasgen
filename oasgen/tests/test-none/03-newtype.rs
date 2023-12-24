use oasgen::OaSchema;
use serde::{Deserialize, Serialize};

#[derive(OaSchema, Serialize, Deserialize)]
pub struct IntegerNewType(i32);

#[derive(OaSchema, Serialize, Deserialize)]
pub struct Struct {
    test: i32,
}

#[derive(OaSchema, Serialize, Deserialize)]
pub struct StructNewType(Struct);

#[derive(OaSchema, Serialize, Deserialize)]
pub struct Foo {
    id: IntegerNewType,
    #[oasgen(inline)]
    prop_a: Struct,
    #[oasgen(inline)]
    prop_b: StructNewType,
    #[oasgen(skip)]
    prop_c: StructNewType,
}

fn main() {
    use pretty_assertions::assert_eq;
    let schema = Foo::schema();
    let spec = serde_yaml::to_string(&schema).unwrap();
    assert_eq!(spec.trim(), include_str!("03-newtype.yaml"));
}
