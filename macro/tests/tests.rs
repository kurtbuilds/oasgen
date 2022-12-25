#[test]
fn run_tests() {
    let t = trybuild::TestCases::new();
    t.pass("tests/01-hello.rs");
}