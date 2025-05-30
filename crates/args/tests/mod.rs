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

#[test]
fn precision_equivalence() {
    let str = "x is 0.01000";

    assert_evaluate!(str, "{0} is {1:.5}", "x", 0.01);
    assert_evaluate!(str, "{1} is {2:.0$}", 5, "x", 0.01);
    assert_evaluate!(str, "{0} is {2:.1$}", "x", 5, 0.01);
    assert_evaluate!(str, "{} is {:.*}", "x", 5, 0.01);
    assert_evaluate!(str, "{1} is {2:.*}", 5, "x", 0.01);
    assert_evaluate!(str, "{} is {2:.*}", "x", 5, 0.01);
    assert_evaluate!(str, "{} is {number:.prec$}", "x", prec = 5, number = 0.01);
}

#[test]
fn precision_inequivalence() {
    assert_evaluate!(
        "Hello, `1234.560` has 3 fractional digits",
        "{}, `{name:.*}` has 3 fractional digits",
        "Hello",
        3,
        name = 1234.56
    );

    assert_evaluate!(
        "Hello, `123` has 3 characters",
        "{}, `{name:.*}` has 3 characters",
        "Hello",
        3,
        name = "1234.56"
    );

    assert_evaluate!(
        "Hello, `     123` has 3 right-aligned characters",
        "{}, `{name:>8.*}` has 3 right-aligned characters",
        "Hello",
        3,
        name = "1234.56"
    );
}

#[test]
fn precision_rounds_half_to_even() {
    assert_evaluate!("1.234e4", "{0:.1$e}", 12345, 3);
    assert_evaluate!("1.236e4", "{0:.1$e}", 12355, 3);
}

#[test]
fn width_includes_precision_and_exp() {
    assert_evaluate!("+001.01230e1", "{n:+0w$.p$e}", n = 10.123, w = 12, p = 5);
}

#[test]
fn width_equivalence() {
    let str = "x    ";
    assert_evaluate!(str, "{:5}", "x");
    assert_evaluate!(str, "{:1$}", "x", 5);
    assert_evaluate!(str, "{1:0$}", 5, "x");
    assert_evaluate!(str, "{:width$}", "x", width = 5);
}

#[test]
fn default_width_align() {
    // left-aligned for non-numerics
    assert_evaluate!("x    ", "{:5}", "x");
    // right-aligned for numerics
    assert_evaluate!("    11", "{:5}", 11);
}

#[test]
fn zero_overwrites_fill() {
    assert_evaluate!("00001", "{:-<05}", 1);
}

#[test]
fn zero_ignored_if_non_numeric() {
    assert_evaluate!("x----", "{:-<05}", "x");
}

#[test]
fn fill_align() {
    assert_evaluate!("x    ", "{:<5}", "x");
    assert_evaluate!("x----", "{:-<5}", "x");
    assert_evaluate!("  x  ", "{:^5}", "x");
    assert_evaluate!("    x", "{:>5}", "x");
}

#[test]
fn zero_padding() {
    assert_evaluate!("001", "{:03}", 1);
    assert_evaluate!("-01", "{:03}", -1);
    assert_evaluate!("011", "{:03}", 11);
    assert_evaluate!("-11", "{:03}", -11);
    assert_evaluate!("1111", "{:03}", 1111);
    assert_evaluate!("-1111", "{:03}", -1111);
}

#[test]
fn sign() {
    assert_evaluate!("+1", "{:+}", 1);
    assert_evaluate!("+0", "{:+}", 0);
    assert_evaluate!("-1", "{:+}", -1);
}

#[test]
fn zero_padding_with_sign() {
    assert_evaluate!("+01", "{:+03}", 1);
    assert_evaluate!("-01", "{:+03}", -1);
}

#[test]
fn sign_positive_for_hex() {
    assert_evaluate!("+0xfffffffe", "{:+#x}", -2);
}

#[test]
fn alternate_form_zero_padding() {
    assert_evaluate!("0x00000f", "{:#08x}", 15);
    assert_evaluate!("0000000f", "{:08x}", 15);
    assert_evaluate!("---0xf", "{:->#6x}", 15);
}

#[test]
fn bool_format() {
    assert_evaluate!("true", "{}", true);
    assert_evaluate!("false", "{}", false);
    // debug doesn't matter
    assert_evaluate!("true", "{:?}", true);
}

#[test]
fn char_format() {
    assert_evaluate!("x", "{}", 'x');
    assert_evaluate!("'x'", "{:?}", 'x');
}

#[test]
fn float_format() {
    // Display removes decimals if all zeros
    assert_evaluate!("1", "{}", 1.000);
    assert_evaluate!("1.01", "{}", 1.01);
    assert_evaluate!("1.000", "{:?}", 1.000);
}

#[test]
fn tuple_debug() {
    assert_evaluate!("(3, 4)", "{:?}", (3, 4));
}

#[test]
fn tuple_debug_passes_format_options() {
    assert_evaluate!("(\n\t0x1--,\n\t0x1--,\n)", "{:-<#5x?}", (1, 1));
    // multiple levels even
    assert_evaluate!(
        "(\n(\n\t\t0x1--,\n\t\t0x2--,\n\t),\n\t0x3--,\n)",
        "{:-<#5x?}",
        ((1, 2), 3)
    );
}

#[test]
fn tuple_debug_pretty() {
    assert_evaluate!("(\n\t3,\n\t4,\n)", "{:?}", (3, 4));
}

// TODO:
// pointer string permutations?
//
// list_debug
// list_debug_pretty

// type_debug
// type_debug_pretty
