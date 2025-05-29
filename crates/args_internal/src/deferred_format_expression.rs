use crate::*;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct DeferredFormatExpression<'a> {
    #[cfg_attr(feature = "serde", serde(borrow))]
    pub(crate) format_string: FormatString<'a>,
    #[cfg_attr(feature = "serde", serde(borrow))]
    pub(crate) provided_args: DeferredArgs<'a>,
}

impl<'a> DeferredFormatExpression<'a> {
    pub fn format_string(&self) -> &FormatString {
        &self.format_string
    }

    pub fn provided_args(&self) -> &DeferredArgs {
        &self.provided_args
    }

    #[doc(hidden)]
    pub unsafe fn new_unchecked(format_string: FormatString<'a>, provided_args: DeferredArgs<'a>) -> Self {
        Self { format_string, provided_args }
    }
}

#[cfg(feature = "syn")]
impl ::syn::parse::Parse for DeferredFormatExpression<'static> {
    fn parse(input: ::syn::parse::ParseStream) -> ::syn::Result<Self> {
        input.parse::<FormatExpression>().map(Into::into)
    }
}

#[cfg(feature = "quote")]
impl quote::ToTokens for DeferredFormatExpression<'_> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let format_string = &self.format_string;
        let provided_args = &self.provided_args;

        const DOC_MESSAGE: &str = "SAFETY: values provided from a validated `DeferredFormatExpression`";

        let format_expression_tokens = quote::quote! {
            unsafe {
                #[doc = #DOC_MESSAGE]
                ::redefmt_args::DeferredFormatExpression::new_unchecked(
                    #format_string,
                    #provided_args
                )
            }
        };

        tokens.extend(format_expression_tokens);
    }
}
