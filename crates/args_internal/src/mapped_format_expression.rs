use crate::*;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct MappedFormatExpression<'a> {
    #[cfg_attr(feature = "serde", serde(borrow))]
    pub(crate) format_string: FormatString<'a>,
    #[cfg_attr(feature = "serde", serde(borrow))]
    pub(crate) provided_args: ProvidedArgsMapping<'a>,
}

impl<'a> MappedFormatExpression<'a> {
    pub fn format_string(&self) -> &FormatString {
        &self.format_string
    }

    pub fn provided_args(&self) -> &ProvidedArgsMapping {
        &self.provided_args
    }

    #[doc(hidden)]
    pub unsafe fn new_unchecked(format_string: FormatString<'a>, provided_args: ProvidedArgsMapping<'a>) -> Self {
        Self { format_string, provided_args }
    }
}

#[cfg(feature = "syn")]
impl ::syn::parse::Parse for MappedFormatExpression<'static> {
    fn parse(input: ::syn::parse::ParseStream) -> ::syn::Result<Self> {
        input.parse::<FormatExpression>().map(Into::into)
    }
}

#[cfg(feature = "quote")]
impl quote::ToTokens for MappedFormatExpression<'_> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let format_string = &self.format_string;
        let provided_args = &self.provided_args;

        const DOC_MESSAGE: &str = "SAFETY: values provided from a validated `MappedFormatExpression`";

        let format_expression_tokens = quote::quote! {
            unsafe {
                #[doc = #DOC_MESSAGE]
                ::redefmt_args::MappedFormatExpression::new_unchecked(
                    #format_string,
                    #provided_args
                )
            }
        };

        tokens.extend(format_expression_tokens);
    }
}
