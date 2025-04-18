/// https://doc.rust-lang.org/std/fmt/index.html#formatting-traits
pub enum FormatTrait {
    /// ''
    Display,
    /// '?'
    Debug,
    /// 'x?'
    DebugLowerHex,
    /// 'X?'
    DebugUpperHex,
    /// 'o'
    Octal,
    /// 'x'
    LowerHex,
    /// 'X'
    UpperHex,
    /// 'p'
    Pointer,
    /// 'b'
    Binary,
    /// 'e'
    LowerExp,
    /// 'E'
    UpperExp,
}
