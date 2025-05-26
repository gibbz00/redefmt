#[derive(redefmt::Format)]
struct Foo();

#[derive(redefmt::Format)]
struct Bar(Foo, Foo);

fn main() {}
