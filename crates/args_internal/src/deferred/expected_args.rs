use alloc::vec::Vec;

use crate::*;

#[derive(Debug, Default, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct DeferredExpectedArgs<'a> {
    pub positional: usize,
    #[cfg_attr(feature = "serde", serde(borrow))]
    // Order does not matter because `ArgumentResolver::resolve` will have disambiguated
    // the corresponding format string arguments from positional into named
    // arguments.
    //
    // Could technically also made into a usize representing the expected named
    // argument count.
    pub named: Vec<AnyIdentifier<'a>>,
}

impl<'a> DeferredExpectedArgs<'a> {
    pub fn count(&self) -> usize {
        self.positional + self.named.len()
    }
}

#[cfg(feature = "quote")]
impl quote::ToTokens for DeferredExpectedArgs<'_> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        use quote::quote;

        let positional = self.positional;
        let named = self.named.iter();

        let provided_args_tokens = quote! {
            ::redefmt_args::deferred::DeferredExpectedArgs {
                positional: #positional,
                named: [#(#named),*].into_iter().collect(),
            }
        };

        tokens.extend(provided_args_tokens);
    }
}
