#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct DeferredExpectedArgs {
    pub positional: usize,
    pub named: usize,
}

impl DeferredExpectedArgs {
    pub fn count(&self) -> usize {
        self.positional + self.named
    }
}

#[cfg(feature = "quote")]
impl quote::ToTokens for DeferredExpectedArgs {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        use quote::quote;

        let positional = self.positional;
        let named = self.named;

        let provided_args_tokens = quote! {
            ::redefmt_args::deferred::DeferredExpectedArgs {
                positional: #positional,
                named: #named,
            }
        };

        tokens.extend(provided_args_tokens);
    }
}
