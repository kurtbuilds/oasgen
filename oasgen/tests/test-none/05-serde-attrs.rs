use oasgen::OaSchema;
use serde::{Deserialize, Serialize};

#[derive(OaSchema, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Bar {
    #[serde(rename = "is_renamed")]
    is_required: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    is_not_required: Option<String>,
}

#[derive(OaSchema, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Duration {
    Day,
    #[serde(rename = "renamedWeek")]
    Week,
    Month,
    #[openapi(skip)]
    Year,
}

#[derive(OaSchema, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Foo {
    #[openapi(inline)]
    camel_bar: Bar,
    #[openapi(inline)]
    camel_duration: Duration,
    #[serde(flatten)]
    flattened: Bar,
}

fn main() {
    use pretty_assertions::assert_eq;
    let schema = Foo::schema().unwrap();
    let spec = serde_yaml::to_string(&schema).unwrap();
    assert_eq!(spec.trim(), include_str!("05-serde-attrs.yaml"));
}