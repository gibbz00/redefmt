use crate::*;

/// https://doc.rust-lang.org/std/fmt/index.html#width
pub enum FormatWidth {
    CountLiteral(usize),
    Argument(Argument),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn width_equivalence() {
        // test https://doc.rust-lang.org/std/fmt/index.html#width
    }
}
