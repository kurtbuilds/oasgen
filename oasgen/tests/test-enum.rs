use pretty_assertions::assert_eq;
use oasgen::OaSchema;
use serde::{Deserialize, Serialize};

#[derive(OaSchema, Serialize, Deserialize)]
pub enum Duration {
    Days(u32),
    Months(u32),
}

#[test]
fn test_duration() {
    let schema = Duration::schema();
    let spec = serde_yaml::to_string(&schema).unwrap();
    let output = include_str!("test-enum/duration.yaml");
    assert_eq!(spec, output)
}
