#[test]
fn run_tests() {
    let t = trybuild::TestCases::new();
    t.pass("tests/test-none/01-hello.rs");
    t.pass("tests/test-none/02-required.rs");
    t.pass("tests/test-none/03-newtype.rs");
    t.pass("tests/test-none/04-enum.rs");
    t.pass("tests/test-none/05-serde-attrs.rs");
    t.pass("tests/test-none/06-complex-enum.rs");
    t.pass("tests/test-none/07-ipaddr.rs");
}
