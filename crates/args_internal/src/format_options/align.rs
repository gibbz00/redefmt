/// <https://doc.rust-lang.org/std/fmt/index.html#fillalignment>
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct FormatAlign {
    pub alignment: Alignment,
    #[cfg_attr(feature = "serde", serde(default, skip_serializing_if = "Option::is_none"))]
    pub character: Option<char>,
}

impl FormatAlign {
    pub(crate) const fn new(alignment: Alignment, character: Option<char>) -> Self {
        Self { alignment, character }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
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
