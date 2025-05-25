fn main() {
    // unused named variable 'y'
    redefmt_args::mapped_format_expression!("{}", 1, y = true);
}
