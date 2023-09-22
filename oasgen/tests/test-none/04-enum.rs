use oasgen::OaSchema;
use serde::{Deserialize, Serialize};

#[derive(OaSchema, Serialize, Deserialize)]
pub enum Duration {
    Day,
    Week,
    Month,
    #[openapi(skip)]
    Year,
}

#[derive(OaSchema, Serialize, Deserialize)]
pub struct Foo {
    duration: Duration,
}

fn main() {
    use pretty_assertions::assert_eq;
    let schema = Foo::schema().unwrap();
    let spec = serde_yaml::to_string(&schema).unwrap();
    assert_eq!(spec.trim(), include_str!("04-enum.yaml"));
}
