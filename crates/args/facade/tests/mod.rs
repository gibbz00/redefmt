#![allow(missing_docs)]

use redefmt_args::{
    deferred::{DeferredFormatConfig, DeferredStructVariant, DeferredTypeValue, DeferredTypeVariant, DeferredValue},
    deferred_format,
};

#[test]
fn ui() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/ui/fail/**/*.rs");
    t.pass("tests/ui/pass/**/*.rs");
}

const FORMAT_DEFERRED_CONFIG: DeferredFormatConfig = DeferredFormatConfig {
    allow_non_usize_precision_value: true,
    allow_non_usize_width_value: true,
};

macro_rules! assert_evaluate {
    ($expected_string:expr, $($arg:tt)*) => {
        let (expression, values) = deferred_format!($($arg)*);
        let actual_string = expression.format_deferred(&values, &FORMAT_DEFERRED_CONFIG).unwrap();
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
    assert_evaluate!("", "{:.*}", 0, 'a');
    // string
    assert_evaluate!("a", "{:.*}", 1, "abc");
    // bool
    assert_evaluate!("tru", "{:.*}", 3, true);
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
fn width_counts_chars_not_bytes() {
    assert_evaluate!("🦀🦀🦀 ", "{:4}", "🦀🦀🦀");
}

#[test]
fn width_proceeds_precision() {
    assert_evaluate!("tru--", "{0:-<1$.2$}", true, 5, 3);
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
fn list_empty() {
    let list: [usize; 0] = [];
    assert_evaluate!("[]", "{:?}", list);
    // Pretty adds no newlines if empty
    assert_evaluate!("[]", "{:#?}", list);
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

#[test]
fn unit_struct() {
    let x = mock_unit_struct();
    assert_evaluate!("Foo", "{:?}", x);
}

#[test]
fn unit_struct_pretty() {
    let x = mock_unit_struct();
    assert_evaluate!("Foo", "{:#?}", x);
}

fn mock_unit_struct() -> DeferredValue<'static> {
    DeferredValue::Type(DeferredTypeValue {
        name: "Foo",
        variant: DeferredTypeVariant::Struct(DeferredStructVariant::Unit),
    })
}

#[test]
fn tuple_struct() {
    let x = mock_tuple_struct();
    assert_evaluate!("Bar(true, 1)", "{:?}", x);
}

#[test]
fn tuple_struct_pretty() {
    let x = mock_tuple_struct();
    assert_evaluate!("Bar(\n\ttrue,\n\t1,\n)", "{:#?}", x);
}

fn mock_tuple_struct() -> DeferredValue<'static> {
    DeferredValue::Type(DeferredTypeValue {
        name: "Bar",
        variant: DeferredTypeVariant::Struct(DeferredStructVariant::Tuple(vec![
            DeferredValue::Boolean(true),
            DeferredValue::Usize(1),
        ])),
    })
}

#[test]
fn empty_tuple_struct() {
    let x = mock_empty_tuple_struct();
    // No parentheses printed
    assert_evaluate!("Bar", "{:?}", x);
    assert_evaluate!("Bar", "{:#?}", x);
}

fn mock_empty_tuple_struct() -> DeferredValue<'static> {
    DeferredValue::Type(DeferredTypeValue {
        name: "Bar",
        variant: DeferredTypeVariant::Struct(DeferredStructVariant::Tuple(vec![])),
    })
}

#[test]
fn named_struct() {
    let x = mock_named_struct();
    assert_evaluate!("Baz { x: true, y: 1 }", "{:?}", x);
}

#[test]
fn named_struct_pretty() {
    let x = mock_named_struct();
    assert_evaluate!("Baz {\n\tx: true,\n\ty: 1,\n}", "{:#?}", x);
}

fn mock_named_struct() -> DeferredValue<'static> {
    DeferredValue::Type(DeferredTypeValue {
        name: "Baz",
        variant: DeferredTypeVariant::Struct(DeferredStructVariant::Named(vec![
            ("x", DeferredValue::Boolean(true)),
            ("y", DeferredValue::Usize(1)),
        ])),
    })
}

#[test]
fn empty_named_struct() {
    let x = mock_empty_named_struct();
    // No curly braces printed
    assert_evaluate!("Baz", "{:?}", x);
    assert_evaluate!("Baz", "{:#?}", x);
}

fn mock_empty_named_struct() -> DeferredValue<'static> {
    DeferredValue::Type(DeferredTypeValue {
        name: "Baz",
        variant: DeferredTypeVariant::Struct(DeferredStructVariant::Named(vec![])),
    })
}

#[test]
fn nested_struct() {
    let x = mock_nested_struct();
    assert_evaluate!("Baz { qux: Qux { value: true } }", "{:?}", x);
}

#[test]
fn nested_struct_pretty() {
    let x = mock_nested_struct();
    assert_evaluate!("Baz {\n\tqux: Qux {\n\t\tvalue: true,\n\t},\n}", "{:#?}", x);
}

fn mock_nested_struct() -> DeferredValue<'static> {
    DeferredValue::Type(DeferredTypeValue {
        name: "Baz",
        variant: DeferredTypeVariant::Struct(DeferredStructVariant::Named(vec![(
            "qux",
            DeferredValue::Type(DeferredTypeValue {
                name: "Qux",
                variant: DeferredTypeVariant::Struct(DeferredStructVariant::Named(vec![(
                    "value",
                    DeferredValue::Boolean(true),
                )])),
            }),
        )])),
    })
}

// Enum printing uses struct printing, no need for excessive testing

#[test]
fn enum_format() {
    let x = mock_enum();
    assert_evaluate!("A(true, 1)", "{:?}", x);
}

fn mock_enum() -> DeferredValue<'static> {
    DeferredValue::Type(DeferredTypeValue {
        name: "Foo",
        variant: DeferredTypeVariant::Enum((
            "A",
            DeferredStructVariant::Tuple(vec![DeferredValue::Boolean(true), DeferredValue::Usize(1)]),
        )),
    })
}
