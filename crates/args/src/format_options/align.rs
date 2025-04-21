use crate::*;

/// https://doc.rust-lang.org/std/fmt/index.html#fillalignment
#[derive(Debug, PartialEq)]
pub struct FormatAlign {
    pub(crate) alignment: Alignment,
    pub(crate) character: Option<char>,
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

impl Alignment {
    pub(crate) fn from_char(ch: char) -> Option<Self> {
        match ch {
            '<' => Some(Self::Left),
            '^' => Some(Self::Center),
            '>' => Some(Self::Right),
            _ => None,
        }
    }
}
