/// https://doc.rust-lang.org/std/fmt/index.html#fillalignment
pub struct FormatAlign {
    alignment: Alignment,
    character: Option<char>,
}
pub enum Alignment {
    /// '<'
    Left,
    /// '^'
    Center,
    /// '>'
    Right,
}
