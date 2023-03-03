#[test]
fn run_tests() {
    let t = trybuild::TestCases::new();
    t.pass("tests/test-none/01-hello.rs");
}