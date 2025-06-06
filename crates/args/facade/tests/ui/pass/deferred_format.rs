fn main() {
    let x = 10;
    let y = String::new();
    let (_expression, _args) = redefmt_args::deferred_format!("hello {x} {y} {z}!", z = &y);
}
