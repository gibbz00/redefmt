#![allow(missing_docs)]

#[test]
fn error_ui() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/ui/**/*.rs");
}
