#[derive(redefmt::Format)]
struct Foo;

fn main() {
    let name = "redefmt";
    // captured arg
    redefmt_logger::print!("hello {name}!");
    // using derived format impl
    redefmt_logger::print!("this is {}", Foo);
    // appended line
    redefmt_logger::println!("bye {ending}", ending = '!');
}
