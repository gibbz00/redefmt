fn main() {
    // both
    redefmt_args::deferred_format_expression!("{} {x} {match}", 1, x = "10", match = r#match);
    // without args
    redefmt_args::deferred_format_expression!("y");
}
