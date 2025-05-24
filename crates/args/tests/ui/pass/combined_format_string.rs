fn main() {
    // both
    redefmt_args::combined_format_string!("{} {x} {match}", 1, x = "10", match = r#match);
    // without args
    redefmt_args::combined_format_string!("y");
}
