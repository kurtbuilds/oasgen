#[test]
fn run_tests() {
    let t = trybuild::TestCases::new();
    t.pass("tests/test-actix/01-hello.rs");
}
