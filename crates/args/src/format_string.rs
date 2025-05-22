use alloc::{borrow::Cow, vec::Vec};

use crate::*;

#[derive(Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(transparent))]
pub struct FormatString<'a> {
    #[cfg_attr(feature = "serde", serde(borrow))]
    pub(crate) segments: Vec<FormatStringSegment<'a>>,
}

#[derive(Debug, Clone, Copy, PartialEq, thiserror::Error)]
#[error("found unclosed format strings arguments")]
pub enum FormatStringParseError {
    #[error("no '{{' found for before '}}'")]
    UnmatchedClose,
    #[error("no '}}' found for after '{{'")]
    UnmatchedOpen,
}

impl<'a> FormatString<'a> {
    pub fn parse(str: &'a str) -> Result<Self, ParseError> {
        Self::parse_impl(0, str)
    }

    pub fn owned(&self) -> FormatString<'static> {
        let segments = self.segments.iter().map(FormatStringSegment::owned).collect();
        FormatString { segments }
    }

    #[cfg(feature = "provided-args")]
    pub(crate) fn collect_args_mut(&mut self) -> Vec<&mut FormatArgument<'a>> {
        let mut format_string_args = Vec::new();
        let mut next_index = 0;

        for segment in &mut self.segments {
            if let FormatStringSegment::Format(format_segment) = segment {
                // disambiguate unnamed with indexed
                let arg = format_segment.argument.get_or_insert_with(|| {
                    let current_index = next_index;
                    next_index += 1;
                    FormatArgument::Index(Integer::new(current_index))
                });

                format_string_args.push(arg);

                if let Some(FormatCount::Argument(width_arg)) = &mut format_segment.options.width {
                    format_string_args.push(width_arg);
                }

                if let Some(FormatPrecision::Count(FormatCount::Argument(precision_arg))) =
                    &mut format_segment.options.precision
                {
                    format_string_args.push(precision_arg);
                }
            }
        }

        format_string_args
    }

    fn parse_impl(offset: usize, str: &'a str) -> Result<Self, ParseError> {
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

                    terminate_string_literal_segment(str, &mut segments, last_segment_end, char_index);

                    // empty format argument
                    if next_char == CLOSING_BRACE {
                        segments.push(FormatStringSegment::Format(Default::default()));
                        last_segment_end = Some(next_char_index + 1);
                        continue;
                    }

                    match char_iter.find(|(_, char)| *char == CLOSING_BRACE) {
                        Some((argument_end_index, _)) => {
                            let format_segment = FormatSegment::parse(
                                offset + next_char_index,
                                &str[next_char_index..argument_end_index],
                            )?;

                            segments.push(FormatStringSegment::Format(format_segment));
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
                    segments.push(FormatStringSegment::Literal(Cow::Borrowed(&str[last_end..])));
                }
            }
            None => {
                if !str.is_empty() {
                    segments.push(FormatStringSegment::Literal(Cow::Borrowed(str)));
                }
            }
        }

        return Ok(Self { segments });

        fn terminate_string_literal_segment<'a>(
            initial_str: &'a str,
            segments: &mut Vec<FormatStringSegment<'a>>,
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

                    if matches!(last_segment, FormatStringSegment::Format(_)) && segment_end != current_char_index {
                        segments.push(FormatStringSegment::Literal(Cow::Borrowed(
                            &initial_str[segment_end..current_char_index],
                        )));
                    }
                }
                None => {
                    if current_char_index != 0 {
                        segments.push(FormatStringSegment::Literal(Cow::Borrowed(
                            &initial_str[0..current_char_index],
                        )));
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
        assert_format_string("{}{{", vec![empty_arg_segment(), str_literal_segment("{{")]);
        assert_format_string("{{{}", vec![str_literal_segment("{{"), empty_arg_segment()]);
    }

    #[test]
    fn escaped_closing_brace() {
        assert_format_string_no_args("text}}");
        assert_format_string_no_args("}}text");
        assert_format_string("{}}}", vec![empty_arg_segment(), str_literal_segment("}}")]);
        assert_format_string("}}{}", vec![str_literal_segment("}}"), empty_arg_segment()]);
    }

    #[test]
    fn escaped_braces_pair() {
        assert_format_string_no_args("text {{}}");
        assert_format_string_no_args("{{text}}");
        assert_format_string(
            "{{{}}}",
            vec![
                str_literal_segment("{{"),
                empty_arg_segment(),
                str_literal_segment("}}"),
            ],
        );
    }

    #[test]
    fn unmatched_close_error() {
        assert_close_error("}");
        assert_close_error("x}");

        fn assert_close_error(str: &str) {
            let expected_error = ParseError::new(0, str.len() - 1..str.len(), FormatStringParseError::UnmatchedClose);
            let actual_error = FormatString::parse(str).unwrap_err();

            assert_eq!(expected_error, actual_error);
        }
    }

    #[test]
    fn unmatched_open_error() {
        assert_open_error("{");
        assert_open_error("{x");

        fn assert_open_error(str: &str) {
            let expected_error = ParseError::new_char(0, FormatStringParseError::UnmatchedOpen);
            let actual_error = FormatString::parse(str).unwrap_err();

            assert_eq!(expected_error, actual_error);
        }
    }

    fn assert_format_string(str: &str, segments: Vec<FormatStringSegment>) {
        let expected_format_string = FormatString { segments };

        let actual_format_string = FormatString::parse(str).unwrap();

        assert_eq!(expected_format_string, actual_format_string);
    }

    fn assert_format_string_no_args(str: &str) {
        assert_format_string(str, vec![str_literal_segment(str)]);
    }

    fn str_literal_segment(str: &str) -> FormatStringSegment<'_> {
        FormatStringSegment::Literal(Cow::Borrowed(str))
    }

    fn empty_arg_segment() -> FormatStringSegment<'static> {
        FormatStringSegment::Format(FormatSegment::default())
    }
}
