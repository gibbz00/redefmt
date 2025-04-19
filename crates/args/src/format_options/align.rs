/// https://doc.rust-lang.org/std/fmt/index.html#fillalignment
#[derive(Debug, PartialEq)]
pub struct FormatAlign {
    alignment: Alignment,
    character: Option<char>,
}

#[derive(Debug, PartialEq)]
pub enum Alignment {
    /// '<'
    Left,
    /// '^'
    Center,
    /// '>'
    Right,
}
