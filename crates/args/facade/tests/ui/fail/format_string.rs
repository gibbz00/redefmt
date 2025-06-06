fn main() {
    // raw identifiers not allowed
    redefmt_args::format_string!("{r#x}");
}
