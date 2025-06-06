#[derive(Debug, Clone, Copy, PartialEq, thiserror::Error)]
pub enum IdentifierParseError {
    #[error("identifier may not be empty")]
    Empty,
    #[error("first (if only character) may not begin with a underscore")]
    Underscore,
    #[error("‌zero width unicode characters (U+200C and U+200D) aren't not allowed")]
    ZeroWidth,
    #[error("invalid XID_Start character")]
    InvalidStartCharacter,
    #[error("invalid XID_Continue character")]
    InvalidContinueCharacter,
    // IMPROVEMENT: not creatable from `RawableIdentifier`
    #[error("raw identifiers are not allowed in format strings")]
    RawIdent,
}
