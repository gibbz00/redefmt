use alloc::{
    string::{String, ToString},
    vec::Vec,
};
use core::{ops::Range, str::FromStr};

use crate::*;

#[derive(Debug, PartialEq)]
enum FormatStringSegment {
    StringLiteral(Range<usize>),
    FormatArgument(FormatArgument),
}

#[derive(Debug, PartialEq)]
pub enum FormatStringParseError {
    /// No '{' found for before '}'
    UnmatchedClose,
    /// No '}' found for after '{'
    UnmatchedOpen,
}

#[derive(Debug, PartialEq)]
pub struct FormatString {
    // IMPROVEMENT: borrowed?
    inner: String,
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

impl FromStr for FormatString {
    type Err = ParseError;

    fn from_str(str: &str) -> Result<Self, Self::Err> {
        FormatString::parse(0, str)
    }
}

impl Parse for FormatString {
    fn parse(offset: usize, str: &str) -> Result<Self, ParseError> {
        const OPENING_BRACE: char = '{';
        const CLOSING_BRACE: char = '}';

        let mut segments = Vec::new();
        let mut char_iter = str.char_indices();
        // index exclusive
        let mut last_segment_end = None;

        while let Some((char_index, char)) = char_iter.next() {
            match char {
                OPENING_BRACE => {
                    let Some((next_char_index, next_char)) = char_iter.next() else {
                        return Err(ParseError::new_char(
                            offset + char_index,
                            FormatStringParseError::UnmatchedOpen,
                        ));
                    };

                    // string escape
                    if next_char == OPENING_BRACE {
                        continue;
                    }

                    terminate_string_literal_segment(&mut segments, last_segment_end, char_index);

                    // empty format argument
                    if next_char == CLOSING_BRACE {
                        segments.push(FormatStringSegment::FormatArgument(Default::default()));
                        last_segment_end = Some(next_char_index + 1);
                        continue;
                    }

                    match char_iter.find(|(_, char)| *char == CLOSING_BRACE) {
                        Some((argument_end_index, _)) => {
                            let format_argument = FormatArgument::parse(
                                offset + next_char_index,
                                &str[next_char_index..argument_end_index],
                            )?;

                            segments.push(FormatStringSegment::FormatArgument(format_argument));
                            last_segment_end = Some(argument_end_index + 1);
                        }
                        None => {
                            return Err(ParseError::new_char(
                                offset + char_index,
                                FormatStringParseError::UnmatchedOpen,
                            ));
                        }
                    }
                }
                CLOSING_BRACE => {
                    if char_iter.next().is_none_or(|(_, next_char)| next_char != CLOSING_BRACE) {
                        return Err(ParseError::new_char(
                            offset + char_index,
                            FormatStringParseError::UnmatchedClose,
                        ));
                    }
                }
                _ => {}
            }
        }

        // save final string segment, if any
        match last_segment_end {
            Some(last_end) => {
                if last_end != str.len() {
                    segments.push(FormatStringSegment::StringLiteral(last_end..str.len()));
                }
            }
            None => {
                if !str.is_empty() {
                    segments.push(FormatStringSegment::StringLiteral(0..str.len()));
                }
            }
        }

        return Ok(Self { segments, inner: str.to_string() });

        fn terminate_string_literal_segment(
            segments: &mut Vec<FormatStringSegment>,
            last_segment_end: Option<usize>,
            current_char_index: usize,
        ) {
            // conditionals exist to avoid filling "{}{}" with an empty string literals before and inbetween
            // no need to update last_segment_end, handled when inserting the upcoming format argument
            match last_segment_end {
                Some(segment_end) => {
                    let last_segment = segments
                        .last()
                        .expect("registered last segment end without pushing to segments buffer");

                    if matches!(last_segment, FormatStringSegment::FormatArgument(_))
                        && segment_end != current_char_index
                    {
                        segments.push(FormatStringSegment::StringLiteral(segment_end..current_char_index));
                    }
                }
                None => {
                    if current_char_index != 0 {
                        segments.push(FormatStringSegment::StringLiteral(0..current_char_index));
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use alloc::vec;

    use super::*;

    #[test]
    fn empty() {
        assert_format_string("", vec![]);
    }

    #[test]
    fn avoids_excessive_string_segments() {
        // none before, inbetween, and after
        assert_format_string("{}{}", vec![empty_arg_segment(), empty_arg_segment()]);
    }

    #[test]
    fn escaped_opening_brace() {
        assert_format_string_no_args("text{{");
        assert_format_string_no_args("{{text");
        assert_format_string(
            "{}{{",
            vec![empty_arg_segment(), FormatStringSegment::StringLiteral(2..4)],
        );
        assert_format_string(
            "{{{}",
            vec![FormatStringSegment::StringLiteral(0..2), empty_arg_segment()],
        );
    }

    #[test]
    fn escaped_closing_brace() {
        assert_format_string_no_args("text}}");
        assert_format_string_no_args("}}text");
        assert_format_string(
            "{}}}",
            vec![empty_arg_segment(), FormatStringSegment::StringLiteral(2..4)],
        );
        assert_format_string(
            "}}{}",
            vec![FormatStringSegment::StringLiteral(0..2), empty_arg_segment()],
        );
    }

    #[test]
    fn escaped_braces_pair() {
        assert_format_string_no_args("text {{}}");
        assert_format_string_no_args("{{text}}");
        assert_format_string(
            "{{{}}}",
            vec![
                FormatStringSegment::StringLiteral(0..2),
                empty_arg_segment(),
                FormatStringSegment::StringLiteral(4..6),
            ],
        );
    }

    #[test]
    fn unmatched_close_error() {
        assert_close_error("}");
        assert_close_error("x}");

        fn assert_close_error(str: &str) {
            let expected_error = ParseError::new(0, str.len() - 1..str.len(), FormatStringParseError::UnmatchedClose);
            let actual_error = FormatString::parse(0, str).unwrap_err();

            assert_eq!(expected_error, actual_error);
        }
    }

    #[test]
    fn unmatched_open_error() {
        assert_open_error("{");
        assert_open_error("{x");

        fn assert_open_error(str: &str) {
            let expected_error = ParseError::new_char(0, FormatStringParseError::UnmatchedOpen);
            let actual_error = FormatString::parse(0, str).unwrap_err();

            assert_eq!(expected_error, actual_error);
        }
    }

    fn assert_format_string(str: &str, segments: Vec<FormatStringSegment>) {
        let expected_format_string = FormatString { inner: str.to_string(), segments };

        let actual_format_string = str.parse().unwrap();

        assert_eq!(expected_format_string, actual_format_string);
    }

    fn assert_format_string_no_args(str: &str) {
        assert_format_string(str, vec![FormatStringSegment::StringLiteral(0..str.len())]);
    }

    fn empty_arg_segment() -> FormatStringSegment {
        FormatStringSegment::FormatArgument(FormatArgument::default())
    }
}
