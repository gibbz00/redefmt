fn main() {
    // simple
    redefmt_args::format_string!("hello {}!");

    // all options combined
    redefmt_args::format_string!("{:🦀>+#0x$.x$x?}!");

    // different identifiers
    redefmt_args::format_string!("{}");
    redefmt_args::format_string!("{0}");
    redefmt_args::format_string!("{x}");
}
