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
    assert_evaluate!("1.0", "{:?}", 1.000);
    assert_evaluate!("1.01", "{}", 1.01);
}

#[test]
fn precision_truncates_non_numerics() {
    // char
    assert_evaluate!("", "{:.*}", 0usize, 'a');
    // string
    assert_evaluate!("a", "{:.*}", 1usize, "abc");
    // bool
    assert_evaluate!("tru", "{:.*}", 3usize, true);
}

#[test]
fn precision_equivalence() {
    let str = "x is 0.01000";

    assert_evaluate!(str, "{0} is {1:.5}", "x", 0.01);
    assert_evaluate!(str, "{1} is {2:.0$}", 5usize, "x", 0.01);
    assert_evaluate!(str, "{0} is {2:.1$}", "x", 5usize, 0.01);
    assert_evaluate!(str, "{} is {:.*}", "x", 5usize, 0.01);
    assert_evaluate!(str, "{1} is {2:.*}", 5usize, "x", 0.01);
    assert_evaluate!(str, "{} is {2:.*}", "x", 5usize, 0.01);
    assert_evaluate!(str, "{} is {number:.prec$}", "x", prec = 5usize, number = 0.01);
}

#[test]
fn precision_inequivalence() {
    assert_evaluate!(
        "Hello, `1234.560` has 3 fractional digits",
        "{}, `{name:.*}` has 3 fractional digits",
        "Hello",
        3usize,
        name = 1234.56
    );

    assert_evaluate!(
        "Hello, `123` has 3 characters",
        "{}, `{name:.*}` has 3 characters",
        "Hello",
        3usize,
        name = "1234.56"
    );

    assert_evaluate!(
        "Hello, `     123` has 3 right-aligned characters",
        "{}, `{name:>8.*}` has 3 right-aligned characters",
        "Hello",
        3usize,
        name = "1234.56"
    );
}

#[test]
fn precision_rounds_half_to_even() {
    assert_evaluate!("1.234e4", "{0:.1$e}", 12345, 3usize);
    assert_evaluate!("1.236e4", "{0:.1$e}", 12355, 3usize);
}

#[test]
fn width_includes_precision_and_exp() {
    assert_evaluate!("+001.01230e1", "{n:+0w$.p$e}", n = 10.123, w = 12usize, p = 5usize);
}

#[test]
fn width_equivalence() {
    let str = "x    ";
    assert_evaluate!(str, "{:5}", "x");
    assert_evaluate!(str, "{:1$}", "x", 5usize);
    assert_evaluate!(str, "{1:0$}", 5usize, "x");
    assert_evaluate!(str, "{:width$}", "x", width = 5usize);
}

#[test]
fn width_counts_chars_not_bytes() {
    assert_evaluate!("🦀🦀🦀 ", "{:4}", "🦀🦀🦀");
}

#[test]
fn width_proceeds_precision() {
    assert_evaluate!("tru--", "{0:-<1$.2$}", true, 5usize, 3usize);
}

#[test]
fn explicit_fill() {
    assert_evaluate!("x    ", "{:<5}", "x");
    assert_evaluate!("x----", "{:-<5}", "x");
    assert_evaluate!("  x  ", "{:^5}", "x");
    // uneven center padding cuts from left
    assert_evaluate!(" x  ", "{:^4}", "x");
    assert_evaluate!("    x", "{:>5}", "x");
}

#[test]
fn default_width_align() {
    // left-aligned for non-numerics
    assert_evaluate!("x    ", "{:5}", "x");
    // right-aligned for numerics
    assert_evaluate!("   11", "{:5}", 11);
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
fn zero_overwrites_fill() {
    assert_evaluate!("00001", "{:-<05}", 1);
}

#[test]
fn zero_ignored_if_non_numeric() {
    assert_evaluate!("x----", "{:-<05}", "x");
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
fn tuple_debug() {
    assert_evaluate!("(3, 4)", "{:?}", (3, 4));
}

#[test]
fn tuple_debug_pretty() {
    assert_evaluate!("(\n\t3,\n\t4,\n)", "{:#?}", (3, 4));
}

#[test]
fn tuple_passes_format_options() {
    assert_evaluate!("(\n\t0x1--,\n\t0x1--,\n)", "{:-<#5x?}", (1, 1));
}

#[test]
fn tuple_nested() {
    let str = "((1, 2), (3, (4, 5)), 6)";
    assert_evaluate!(str, "{:?}", ((1, 2), (3, (4, 5)), 6));
}

#[test]
fn tuple_nested_pretty() {
    let str = "(\n\t(\n\t\t1,\n\t\t2,\n\t),\n\t(\n\t\t3,\n\t\t(\n\t\t\t4,\n\t\t\t5,\n\t\t),\n\t),\n\t6,\n)";
    assert_evaluate!(str, "{:#?}", ((1, 2), (3, (4, 5)), 6));
}

#[test]
fn list_nested() {
    let str = "[[1, 2], [3, 4], [5, 6]]";
    assert_evaluate!(str, "{:?}", [[1, 2], [3, 4], [5, 6]]);
}

#[test]
fn list_nested_pretty() {
    let str = "[\n\t[\n\t\t1,\n\t\t2,\n\t],\n\t[\n\t\t3,\n\t\t4,\n\t],\n\t[\n\t\t5,\n\t\t6,\n\t],\n]";
    assert_evaluate!(str, "{:#?}", [[1, 2], [3, 4], [5, 6]]);
}
