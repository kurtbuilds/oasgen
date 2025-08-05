#[test]
fn test_axum() {
    let t = trybuild::TestCases::new();
    t.pass("tests/test-axum/01-hello.rs");
    t.pass("tests/test-axum/02-query.rs");
    t.pass("tests/test-axum/03-path.rs");
    t.pass("tests/test-axum/04-status_code.rs");
}
