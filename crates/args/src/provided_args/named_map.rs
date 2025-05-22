use alloc::vec::Vec;
use core::ops::{Deref, DerefMut};

use crate::*;

type Inner = Vec<(syn::Ident, ProvidedArgValue)>;

/// Sorted collection with unique elements.
///
/// Sorting requirements matters when reusing named arguments for
/// positional arguments.
#[derive(Debug, Default, Clone, PartialEq)]
pub struct ProvidedNamedArgsMap(Inner);

impl ProvidedNamedArgsMap {
    pub(crate) fn from_inner(inner: Inner) -> Self {
        Self(inner)
    }
}

impl Deref for ProvidedNamedArgsMap {
    type Target = Inner;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for ProvidedNamedArgsMap {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[cfg(feature = "provided-args-serde")]
mod serde {
    use ::serde::{Deserialize, Serialize};
    use syn_serde::Syn;

    use super::*;

    impl Serialize for ProvidedNamedArgsMap {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: ::serde::Serializer,
        {
            let iter = self.iter().map(|(ident, value)| (ident.to_adapter(), value));
            serializer.collect_seq(iter)
        }
    }

    impl<'de> Deserialize<'de> for ProvidedNamedArgsMap {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: ::serde::Deserializer<'de>,
        {
            let adapted_vec = Vec::deserialize(deserializer)?;

            let inner = adapted_vec
                .into_iter()
                .map(|(adapted_ident, value)| (syn::Ident::from_adapter(&adapted_ident), value))
                .collect();

            Ok(Self(inner))
        }
    }
}
