use oasgen::OaSchema;
use serde::{Deserialize, Serialize};

#[derive(OaSchema, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Bar {
    is_required: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    is_not_required: Option<String>,
}

#[derive(OaSchema, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Foo {
    prop_a: Bar,
    #[serde(flatten)]
    prop_b: Bar,
}

fn main() {
    use pretty_assertions::assert_eq;
    let schema = Foo::schema().unwrap();
    let spec = serde_yaml::to_string(&schema).unwrap();
    assert_eq!(spec.trim(), include_str!("05-serde-attrs.yaml"));
}
