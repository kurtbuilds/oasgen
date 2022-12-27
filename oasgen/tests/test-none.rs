#![cfg(not(any(feature = "actix")))]
/// Tests when we have no framework activated

#[test]
fn run_tests() {
    let t = trybuild::TestCases::new();
    t.pass("tests/test-none/01-hello.rs");
}