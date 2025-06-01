use alloc::vec::Vec;

use hashbrown::HashMap;

use crate::*;

pub struct DeferredValues<'v> {
    // IMPROVEMENT: Cow rather than require vec?
    pub(crate) positional: Vec<DeferredValue<'v>>,
    pub(crate) named: HashMap<ArgumentIdentifier<'v>, DeferredValue<'v>>,
}

impl<'v> DeferredValues<'v> {
    pub fn new(
        positional: impl IntoIterator<Item = DeferredValue<'v>>,
        named: impl IntoIterator<Item = (AnyIdentifier<'v>, DeferredValue<'v>)>,
    ) -> Self {
        Self {
            positional: positional.into_iter().collect(),
            named: named
                .into_iter()
                .map(|(identifier, value)| (identifier.unraw(), value))
                .collect(),
        }
    }

    pub fn get<'s>(
        &'s self,
        format_argument: &FormatArgument<'s>,
    ) -> Result<&'s DeferredValue<'v>, DeferredFormatError> {
        match format_argument {
            FormatArgument::Identifier(identifier) => self
                .named
                .get(identifier)
                .ok_or_else(|| DeferredFormatError::MissingNamed(identifier.owned())),
            FormatArgument::Index(index) => self
                .positional
                .get(*index)
                .ok_or(DeferredFormatError::PositionalOutOfBounds(*index)),
        }
    }
}
