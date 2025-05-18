#[derive(redefmt::Format)]
enum Foo {}

#[derive(redefmt::Format)]
enum Bar {
    Unit,
}

#[derive(redefmt::Format)]
enum Baz {
    Tuple(Foo, Bar),
    Named { foo: Foo, bar: Bar },
}

fn main() {}
