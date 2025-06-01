use alloc::string::String;

use crate::*;

// Format string has been passed through argument resolver
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct DeferredFormatString<'a> {
    #[cfg_attr(feature = "serde", serde(borrow))]
    pub(crate) format_string: FormatString<'a>,
}

impl<'a> DeferredFormatString<'a> {
    pub fn evaluate<'v>(&self, provided_args: &DeferredProvidedArgs<'v>) -> Result<String, DeferredFormatError> {
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
    pub unsafe fn new_unchecked(format_string: FormatString<'a>) -> Self {
        Self { format_string }
    }
}

#[cfg(feature = "quote")]
impl quote::ToTokens for DeferredFormatString<'_> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let format_string = &self.format_string;

        const DOC_MESSAGE: &str = "SAFETY: values provided from a validated `DeferredFormatExpression`";

        let format_expression_tokens = quote::quote! {
            unsafe {
                #[doc = #DOC_MESSAGE]
                ::redefmt_args::deferred::DeferredFormatString::new_unchecked(
                    #format_string,
                )
            }
        };

        tokens.extend(format_expression_tokens);
    }
}
