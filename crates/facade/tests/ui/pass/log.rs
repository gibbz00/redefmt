#[derive(redefmt::Format)]
struct Foo;

fn main() {
    let name = "redefmt";
    redefmt::log!(redefmt::Level::Info, "hello {name}!");

    // level can be full-blown expression
    redefmt::log!(
        if true {
            redefmt::Level::Info
        } else {
            redefmt::Level::Error
        },
        "{}",
        Foo
    );
}
