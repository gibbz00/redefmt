fn main() {
    redefmt_args::combined_format_string!("{} {x} {match}", 1, x = "10", match = r#match);
}
