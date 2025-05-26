#[derive(redefmt::Format)]
struct Foo;

fn main() {
    let name = "redefmt";
    // captured arg
    redefmt::print!("hello {name}!");
    // using derived format impl
    redefmt::print!("this is {}", Foo);
    // appended line
    redefmt::println!("bye {ending}", ending = '!');
}
