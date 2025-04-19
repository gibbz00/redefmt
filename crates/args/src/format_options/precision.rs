use crate::*;

/// https://doc.rust-lang.org/std/fmt/index.html#precision
pub enum FormatPrecision {
    CountLiteral(usize),
    Argument(Argument),
    /// '*'
    NextArgument,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn precision_equivalence() {
        // test https://doc.rust-lang.org/std/fmt/index.html#precision
    }
}
