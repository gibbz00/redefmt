use core::ops::Range;

use crate::*;

#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum FormatStringParseErrorKind {
    #[error("failed to parse integer")]
    Integer(#[from] core::num::ParseIntError),
    #[error("failed to parse identifier")]
    Identifier(#[from] IdentifierParseError),
    #[error("failed to parse count argument")]
    Count(#[from] FormatCountParseError),
    #[error("failed to parse precision format option")]
    Precision(#[from] FormatPrecisionParseError),
    #[error("failed to parse format trait")]
    Trait(#[from] FormatTraitParseError),
    #[error("found unclosed format strings arguments")]
    String(#[from] FormatStringSegmentError),
}

#[derive(Debug, PartialEq)]
pub struct FormatStringParseError {
    // NOTE: character indices, not byte
    range: Range<usize>,
    kind: FormatStringParseErrorKind,
}

impl FormatStringParseError {
    pub fn kind(&self) -> &FormatStringParseErrorKind {
        &self.kind
    }

    pub(crate) fn new(offset: usize, range: Range<usize>, kind: impl Into<FormatStringParseErrorKind>) -> Self {
        let range = offset + range.start..offset + range.end;
        Self { range, kind: kind.into() }
    }

    pub(crate) fn new_char(offset: usize, kind: impl Into<FormatStringParseErrorKind>) -> Self {
        Self::new(offset, 0..1, kind)
    }

    // A bit hacky, but ok for now.
    #[cfg(feature = "syn")]
    pub(crate) fn pretty_print(&self, format_str: &str) -> alloc::string::String {
        use alloc::string::{String, ToString};
        use core::error::Error;

        let bar_string = {
            let bar_offset = 1 + self.range.start; // + 1 for leading quotation mark
            let bar_len = self.range.end - self.range.start;

            let mut bar_string = String::with_capacity(bar_offset + bar_len);

            (0..bar_offset).for_each(|_| bar_string.push(' '));
            (0..bar_len + 1).for_each(|_| bar_string.push('^'));

            bar_string
        };

        let error_intro = uppercase_first(self.kind.to_string());

        let error_cause = self
            .kind
            .source()
            .map(|source| alloc::format!(", caused by: {source}"))
            .unwrap_or_default();

        return alloc::format!("Invalid format string:\n\n'{format_str}'\n{bar_string}\n{error_intro}{error_cause}.",);

        fn uppercase_first(str: String) -> String {
            let mut c = str.chars();
            match c.next() {
                Some(first) => {
                    let mut new_string = first.to_uppercase().collect::<String>();
                    new_string.extend(c);
                    new_string
                }
                None => String::new(),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[rustfmt::skip]
    const EXPECTED_PRETTY_ERROR: &str =
"Invalid format string:

'hello {r#x}!'
        ^^^
Failed to parse identifier, caused by: raw identifiers are not allowed in format strings.";

    #[test]
    fn pretty_print_error() {
        let str = "hello {r#x}!";

        let actual_pretty_error = FormatString::parse(str).unwrap_err().pretty_print(str);

        assert_eq!(EXPECTED_PRETTY_ERROR, actual_pretty_error);
    }
}
