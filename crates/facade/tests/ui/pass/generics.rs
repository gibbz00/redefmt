#[derive(redefmt::Format)]
struct Foo<T>(T);

fn main() {
    redefmt::println!("{}", Foo(1));
    redefmt::println!("{}", Foo("x"));
}
