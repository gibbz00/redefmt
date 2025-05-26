#[derive(redefmt::Format)]
struct Foo {}

#[derive(redefmt::Format)]
struct Boo {
    foo_0: Foo,
    foo_1: Foo,
}

fn main() {}
