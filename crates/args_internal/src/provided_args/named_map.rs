use alloc::vec::Vec;
use core::ops::{Deref, DerefMut};

use crate::*;

type Inner<'a> = Vec<(AnyIdentifier<'a>, ProvidedArgValue<'a>)>;

#[derive(Debug, Default, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ProvidedNamedArgsMap<'a>(#[cfg_attr(feature = "serde", serde(borrow))] Inner<'a>);

impl<'a> ProvidedNamedArgsMap<'a> {
    pub(crate) fn from_inner(inner: Inner<'a>) -> Self {
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
