use crate::*;

#[derive(Debug, Default, PartialEq, Eq, Hash, derive_getters::Getters)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct FormatSegment<'a> {
    #[cfg_attr(feature = "serde", serde(default, borrow, skip_serializing_if = "Option::is_none"))]
    pub(crate) argument: Option<FormatArgument<'a>>,
    #[cfg_attr(feature = "serde", serde(borrow))]
    pub(crate) options: FormatOptions<'a>,
}

impl<'a> FormatSegment<'a> {
    /// Context from `FormatString::parse`:
    /// - Does not contain opening and closing braces.
    /// - `str` not empty.
    pub(crate) fn parse(offset: usize, str: &'a str) -> Result<Self, ParseError> {
        let format_segment = match str.find(':') {
            Some(split_index) => {
                let argument_str = &str[0..split_index];

                let argument = match argument_str.is_empty() {
                    true => None,
                    false => Some(FormatArgument::parse(offset, argument_str)?),
                };

                let format_options_str = &str[split_index + 1..].trim_end();
                let options = FormatOptions::parse(offset + split_index + 1, format_options_str)?;

                Self { argument, options }
            }
            None => {
                let argument = FormatArgument::parse(offset, str)?;
                Self { argument: Some(argument), options: Default::default() }
            }
        };

        Ok(format_segment)
    }

    pub(crate) fn owned(&self) -> FormatSegment<'static> {
        let FormatSegment { argument, options } = self;

        FormatSegment {
            argument: argument.as_ref().map(|arg| arg.owned()),
            options: options.owned(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_empty() {
        assert_parse(":", None, Default::default());
    }

    #[test]
    fn trim_whispcace_end() {
        assert_parse(":\t\r\n ", None, Default::default());
    }

    #[test]
    fn parse_with_argument_only() {
        assert_parse("x:", mock_argument(), Default::default());
        assert_parse("x", mock_argument(), Default::default());
    }

    #[test]
    fn parse_with_options_only() {
        assert_parse(":?", None, mock_options());
    }

    #[test]
    fn parse_with_both_argument_and_options() {
        assert_parse("x:?", mock_argument(), mock_options());
    }

    fn assert_parse(str: &str, expected_argument: Option<FormatArgument>, expected_options: FormatOptions) {
        let actual = FormatSegment::parse(0, str).unwrap();
        let expected = FormatSegment { argument: expected_argument, options: expected_options };

        assert_eq!(expected, actual);
    }

    fn mock_argument() -> Option<FormatArgument<'static>> {
        let argument = FormatArgument::parse(0, "x").unwrap();
        Some(argument)
    }

    fn mock_options() -> FormatOptions<'static> {
        FormatOptions::builder().format_trait(FormatTrait::Debug).build()
    }
}
