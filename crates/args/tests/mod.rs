#![allow(missing_docs)]

use redefmt_args::deferred_format;

#[test]
fn ui() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/ui/fail/**/*.rs");
    t.pass("tests/ui/pass/**/*.rs");
}

macro_rules! assert_evaluate {
    ($expected_string:expr, $($arg:tt)*) => {
        let (expression, args) = deferred_format!($($arg)*);
        let actual_string = expression.evaluate(&args).unwrap();
        assert_eq!($expected_string, actual_string);
    };
}

#[test]
fn empty() {
    assert_evaluate!("", "");
}
