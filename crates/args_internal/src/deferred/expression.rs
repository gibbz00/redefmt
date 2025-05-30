use alloc::string::String;

use crate::*;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct DeferredFormatExpression<'a> {
    #[cfg_attr(feature = "serde", serde(borrow))]
    pub(crate) format_string: FormatString<'a>,
    pub(crate) expected_args: DeferredExpectedArgs,
}

impl<'a> DeferredFormatExpression<'a> {
    pub fn format_string(&self) -> &FormatString {
        &self.format_string
    }

    pub fn expected_args(&self) -> &DeferredExpectedArgs {
        &self.expected_args
    }

    pub fn evaluate<'v>(&self, provided_args: &DeferredProvidedArgs<'v>) -> Result<String, DeferredFormatError> {
        // TODO: check that provided and expected named arg counts match?
        // implicitly creates the rule that no unused named arguments are
        // allowed.

        let mut string = String::new();

        for segment in self.format_string.segments() {
            match segment {
                FormatStringSegment::Literal(literal) => {
                    string.push_str(literal.as_ref());
                }
                FormatStringSegment::Format(segment) => {
                    // IMPROVEMENT: remove `expect` by creating a separate
                    // type for the format arguments resolved by `ArgumentResolver`
                    let argument = segment
                        .argument
                        .as_ref()
                        .expect("argument not disambiguated by argument resolver");

                    provided_args
                        .get(argument)?
                        .evaluate(&mut string, &segment.options, provided_args);
                }
            }
        }

        Ok(string)
    }

    #[doc(hidden)]
    pub unsafe fn new_unchecked(format_string: FormatString<'a>, expected_args: DeferredExpectedArgs) -> Self {
        Self { format_string, expected_args }
    }
}

#[cfg(feature = "syn")]
impl ::syn::parse::Parse for DeferredFormatExpression<'static> {
    fn parse(input: ::syn::parse::ParseStream) -> ::syn::Result<Self> {
        let format_expression = input.parse::<FormatExpression>()?;

        let (deferred_format_expression, _) = format_expression.defer();

        Ok(deferred_format_expression)
    }
}

#[cfg(feature = "quote")]
impl quote::ToTokens for DeferredFormatExpression<'_> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let format_string = &self.format_string;
        let expected_args = &self.expected_args;

        const DOC_MESSAGE: &str = "SAFETY: values provided from a validated `DeferredFormatExpression`";

        let format_expression_tokens = quote::quote! {
            unsafe {
                #[doc = #DOC_MESSAGE]
                ::redefmt_args::deferred::DeferredFormatExpression::new_unchecked(
                    #format_string,
                    #expected_args
                )
            }
        };

        tokens.extend(format_expression_tokens);
    }
}
