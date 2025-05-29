fn main() {
    // unused named variable 'y'
    redefmt_args::deferred_format_expression!("{}", 1, y = true);
}
