use crate::*;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum FormatStringSegment<'a> {
    Literal(FormatLiteral<'a>),
    #[cfg_attr(feature = "serde", serde(borrow))]
    Format(FormatArgumentSegment<'a>),
}

impl FormatStringSegment<'_> {
    pub(crate) fn owned(&self) -> FormatStringSegment<'static> {
        match self {
            FormatStringSegment::Literal(literal) => FormatStringSegment::Literal(literal.owned()),
            FormatStringSegment::Format(segment) => FormatStringSegment::Format(segment.owned()),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, thiserror::Error)]
pub enum FormatStringSegmentError {
    #[error("no '{{' found for before '}}'")]
    UnmatchedClose,
    #[error("no '}}' found for after '{{'")]
    UnmatchedOpen,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct FormatArgumentSegment<'a> {
    #[cfg_attr(feature = "serde", serde(default, borrow, skip_serializing_if = "Option::is_none"))]
    pub argument: Option<FormatArgument<'a>>,
    #[cfg_attr(feature = "serde", serde(borrow))]
    pub options: FormatOptions<'a>,
}

impl<'a> FormatArgumentSegment<'a> {
    /// Context from `FormatString::parse`:
    /// - Does not contain opening and closing braces.
    /// - `str` not empty.
    pub(crate) fn parse(offset: usize, str: &'a str) -> Result<Self, FormatStringParseError> {
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

    pub(crate) fn owned(&self) -> FormatArgumentSegment<'static> {
        let FormatArgumentSegment { argument, options } = self;

        FormatArgumentSegment {
            argument: argument.as_ref().map(|arg| arg.owned()),
            options: options.owned(),
        }
    }
}

#[cfg(feature = "quote")]
mod quote {
    use ::quote::{ToTokens, quote};

    use crate::*;

    impl ToTokens for FormatStringSegment<'_> {
        fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
            let variant_tokens = match self {
                FormatStringSegment::Literal(literal) => quote! { Literal(#literal) },
                FormatStringSegment::Format(format_segment) => quote! { Format(#format_segment) },
            };

            let segment_tokens = quote! {
                ::redefmt_args::format_segment::FormatStringSegment::#variant_tokens
            };

            tokens.extend(segment_tokens);
        }
    }

    impl ToTokens for FormatArgumentSegment<'_> {
        fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
            let argument = quote_utils::PrintOption::new(&self.argument);
            let options = &self.options;

            let segment_tokens = quote! {
                ::redefmt_args::format_segment::FormatArgumentSegment {
                    argument: #argument,
                    options: #options,
                }
            };

            tokens.extend(segment_tokens);
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
        let actual = FormatArgumentSegment::parse(0, str).unwrap();
        let expected = FormatArgumentSegment { argument: expected_argument, options: expected_options };

        assert_eq!(expected, actual);
    }

    fn mock_argument() -> Option<FormatArgument<'static>> {
        let argument = FormatArgument::parse(0, "x").unwrap();
        Some(argument)
    }

    fn mock_options() -> FormatOptions<'static> {
        FormatOptions { format_trait: FormatTrait::Debug, ..Default::default() }
    }
}
