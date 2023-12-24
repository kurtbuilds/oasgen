use oasgen::OaSchema;
use serde::{Deserialize, Serialize};

#[derive(OaSchema, Serialize, Deserialize)]
pub enum Duration {
    Day,
    Week,
    Month,
    #[oasgen(skip)]
    Year,
}

#[derive(OaSchema, Serialize, Deserialize)]
pub struct Foo {
    #[oasgen(inline)]
    duration: Duration,
}

fn main() {
    use pretty_assertions::assert_eq;
    let schema = Foo::schema();
    let spec = serde_yaml::to_string(&schema).unwrap();
    assert_eq!(spec.trim(), include_str!("04-enum.yaml"));
}
