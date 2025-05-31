use alloc::string::String;

use crate::*;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct DeferredFormatExpression<'a> {
    #[cfg_attr(feature = "serde", serde(borrow))]
    pub(crate) format_string: FormatString<'a>,
    pub(crate) expected_arg_count: usize,
}

impl<'a> DeferredFormatExpression<'a> {
    pub fn expected_arg_count(&self) -> usize {
        self.expected_arg_count
    }

    pub fn evaluate<'v>(&self, provided_args: &DeferredProvidedArgs<'v>) -> Result<String, DeferredFormatError> {
        // TODO: Check that provided and expected arg counts match? implicitly
        // creates the rule that no unused named arguments are allowed. Should
        // be done individually for positional and named, which means that
        // `Self` can't store them in a combined sum.

        let mut string = String::new();

        for segment in self.format_string.segments() {
            match segment {
                FormatStringSegment::Literal(literal) => {
                    string.push_str(literal.as_ref());
                }
                FormatStringSegment::Format(segment) => {
                    // IMPROVEMENT: remove `expect` by creating a separate
                    // type for the format arguments resolved by `ArgumentResolver`?
                    let argument = segment
                        .argument
                        .as_ref()
                        .expect("argument not disambiguated by argument resolver");

                    provided_args
                        .get(argument)?
                        .evaluate(&mut string, &segment.options, provided_args)?;
                }
            }
        }

        Ok(string)
    }

    #[doc(hidden)]
    pub unsafe fn new_unchecked(format_string: FormatString<'a>, expected_arg_count: usize) -> Self {
        Self { format_string, expected_arg_count }
    }
}

#[cfg(feature = "quote")]
impl quote::ToTokens for DeferredFormatExpression<'_> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let format_string = &self.format_string;
        let expected_arg_count = &self.expected_arg_count;

        const DOC_MESSAGE: &str = "SAFETY: values provided from a validated `DeferredFormatExpression`";

        let format_expression_tokens = quote::quote! {
            unsafe {
                #[doc = #DOC_MESSAGE]
                ::redefmt_args::deferred::DeferredFormatExpression::new_unchecked(
                    #format_string,
                    #expected_arg_count
                )
            }
        };

        tokens.extend(format_expression_tokens);
    }
}
