use crate::*;

#[derive(Debug, Default, PartialEq)]
pub struct FormatArgument {
    argument: Option<Argument>,
    options: FormatOptions,
}

impl Parse for FormatArgument {
    // Context from `FormatString::parse`:
    // - Does not contain opening and closing braces.
    // - `str` not empty.
    fn parse(offset: usize, str: &str) -> Result<Self, ParseError> {
        let format_argument = match str.find(':') {
            Some(split_index) => {
                let argument_str = &str[0..split_index];

                let argument = match argument_str.is_empty() {
                    true => None,
                    false => Some(Argument::parse(offset, argument_str)?),
                };

                let format_options_str = &str[split_index + 1..];
                let options = FormatOptions::parse(offset + split_index + 1, format_options_str)?;

                Self { argument, options }
            }
            None => {
                let argument = Argument::parse(offset, str)?;
                Self { argument: Some(argument), options: Default::default() }
            }
        };

        Ok(format_argument)
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

    fn assert_parse(str: &str, expected_argument: Option<Argument>, expected_options: FormatOptions) {
        let actual = FormatArgument::parse(0, str).unwrap();
        let expected = FormatArgument { argument: expected_argument, options: expected_options };

        assert_eq!(expected, actual);
    }

    fn mock_argument() -> Option<Argument> {
        let argument = Argument::parse(0, "x").unwrap();
        Some(argument)
    }

    fn mock_options() -> FormatOptions {
        FormatOptions::builder().format_trait(FormatTrait::Debug).build()
    }
}
