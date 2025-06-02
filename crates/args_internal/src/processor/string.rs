use alloc::string::String;

use crate::*;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ProcessedFormatString<'a>(#[cfg_attr(feature = "serde", serde(borrow))] pub(crate) FormatString<'a>);

impl<'a> ProcessedFormatString<'a> {
    pub fn format_deferred<'v>(&self, deferred_values: &DeferredValues<'v>) -> Result<String, DeferredFormatError> {
        let mut string_buffer = String::new();

        for segment in self.0.segments() {
            match segment {
                FormatStringSegment::Literal(literal) => {
                    string_buffer.push_str(literal.as_ref());
                }
                FormatStringSegment::Format(segment) => {
                    // IMPROVEMENT: remove `expect` by creating a separate
                    // type for the format arguments resolved by `ArgumentResolver`?
                    let argument = segment
                        .argument
                        .as_ref()
                        .expect("argument not disambiguated by argument resolver");

                    let options = ResolvedFormatOptions::new(&segment.options, deferred_values)?;

                    deferred_values
                        .get(argument)?
                        .format_deferred(&mut string_buffer, &options)?;
                }
            }
        }

        Ok(string_buffer)
    }

    #[doc(hidden)]
    pub unsafe fn new_unchecked(format_string: FormatString<'a>) -> Self {
        Self(format_string)
    }
}

#[cfg(feature = "quote")]
impl quote::ToTokens for ProcessedFormatString<'_> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let format_string = &self.0;

        const DOC_MESSAGE: &str = "SAFETY: values provided from a validated `ProcessedFormatString`";

        let format_expression_tokens = quote::quote! {
            unsafe {
                #[doc = #DOC_MESSAGE]
                ::redefmt_args::processor::ProcessedFormatString::new_unchecked(
                    #format_string,
                )
            }
        };

        tokens.extend(format_expression_tokens);
    }
}
