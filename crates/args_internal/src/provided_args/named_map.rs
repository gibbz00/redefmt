use alloc::vec::Vec;
use core::ops::{Deref, DerefMut};

use crate::*;

type Inner<'a> = Vec<(AnyIdentifier<'a>, ProvidedArgValue<'a>)>;

#[derive(Debug, Default, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ProvidedNamedArgsMap<'a>(#[cfg_attr(feature = "serde", serde(borrow))] Inner<'a>);

impl<'a> ProvidedNamedArgsMap<'a> {
    pub fn new(inner: Inner<'a>) -> Self {
        Self(inner)
    }
}

impl<'a> Deref for ProvidedNamedArgsMap<'a> {
    type Target = Inner<'a>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'a> DerefMut for ProvidedNamedArgsMap<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[cfg(feature = "quote")]
impl quote::ToTokens for ProvidedNamedArgsMap<'_> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        use quote::quote;

        let elements = self.iter().map(|(identifier, value)| quote! { (#identifier, #value) });

        let map_tokens = quote! {
            ::redefmt_args::provided_args::ProvidedNamedArgsMap::new([ #(#elements),* ].into_iter().collect())
        };

        tokens.extend(map_tokens);
    }
}
