use oasgen::OaSchema;
use serde::{Deserialize, Serialize};

pub struct Bar {
    test: String,
}

#[derive(OaSchema, Serialize, Deserialize)]
pub enum ExternallyTagged {
    A(i32),
    B,
    C { test: i32 },
    D(Bar),
    E,
}

#[derive(OaSchema, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum InternallyTagged {
    // A(i32), internally tagged does not support tuple variants that do not contain a struct
    B,
    C { test: i32 },
    D(Bar),
    E,
}

#[derive(OaSchema, Serialize, Deserialize)]
#[serde(tag = "t", content = "c")]
pub enum AdjacentlyTagged {
    A(i32),
    B,
    C { test: i32 },
    D(Bar),
    E,
}

#[derive(OaSchema, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Untagged {
    A(i32),
    B,
    C { test: i32 },
    D(Bar),
    E,
}

#[derive(OaSchema, Serialize, Deserialize)]
pub struct Foo {
    externally_tagged: ExternallyTagged,
    internally_tagged: InternallyTagged,
    adjacently_tagged: AdjacentlyTagged,
    untagged: Untagged,
}

fn main() {
    use pretty_assertions::assert_eq;
    let schema = Foo::schema().unwrap();
    let spec = serde_yaml::to_string(&schema).unwrap();
    assert_eq!(spec.trim(), include_str!("06-serde-attrs.yaml"));
}
