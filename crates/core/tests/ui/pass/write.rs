struct Foo;

impl redefmt::Format for Foo {
    fn fmt(&self, random_ident: &mut redefmt::Formatter) {
        let name = "redefmt";
        redefmt::write!(random_ident, "hello {name}!");
        redefmt::writeln!(random_ident, "hello {name}!");
    }
}

fn main() {}
