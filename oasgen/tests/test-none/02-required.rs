use oasgen::{OaSchema};
use serde::{Deserialize, Serialize};

#[derive(OaSchema, Serialize, Deserialize)]
pub struct Foo {
    is_required: String,
    is_nullable: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    is_not_required: Option<String>,
    #[allow(dead_code)]
    #[serde(skip)]
    is_not_on_schema: String,
    #[oasgen(skip)]
    is_also_not_on_schema: String,
}

fn main() {
    use pretty_assertions::assert_eq;
    let schema = Foo::schema().unwrap();
    let spec = serde_yaml::to_string(&schema).unwrap();
    assert_eq!(spec.trim(), include_str!("02-required.yaml"));
}