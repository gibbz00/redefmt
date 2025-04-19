use alloc::{string::String, vec::Vec};

use crate::*;

pub struct FormatString {
    segments: Vec<FormatStringSegment>,
}

impl FormatString {
    // TODO:
    // https://doc.rust-lang.org/std/fmt/index.html#positional-parameters
    // should also resolve the parameters for width and precision
    pub fn resolve_parameters() {
        todo!()
    }
}

enum FormatStringSegment {
    // IMPROVEMENT: borrowed?
    StringLiteral(String),
    FormatArgument(FormatArgument),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn escaped_opening_brace() {
        // "text{{"
        // "{{text"
        // "{}{{"
        // "{{{}"
    }

    #[test]
    fn escaped_closing_brace() {
        // "text}}"
        // "}}text"
        // "{}}}"
        // "}}{}"
    }

    #[test]
    fn escaped_braces_pair() {
        // "text {{}}"
        // "{{text}}"
        // "{{{}}}"
    }
}
