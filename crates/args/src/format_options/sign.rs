/// https://doc.rust-lang.org/std/fmt/index.html#sign0
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Sign {
    /// '+'
    Plus,
    /// '-'
    Minus,
}
