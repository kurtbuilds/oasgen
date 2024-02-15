#[test]
fn run_tests() {
    let t = trybuild::TestCases::new();
    t.pass("tests/test-axum/01-hello.rs");
    t.pass("tests/test-axum/02-query.rs");
}