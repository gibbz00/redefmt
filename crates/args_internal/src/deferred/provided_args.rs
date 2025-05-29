use hashbrown::HashMap;

use crate::*;

pub struct DeferredProvidedArgs<'a, 'v> {
    pub positional: &'a [DeferredFormatValue<'v>],
    pub named: HashMap<ArgumentIdentifier<'v>, DeferredFormatValue<'v>>,
}

impl<'a, 'v> DeferredProvidedArgs<'a, 'v> {
    pub fn new(
        positional: &'a [DeferredFormatValue<'v>],
        named: impl IntoIterator<Item = (AnyIdentifier<'v>, DeferredFormatValue<'v>)>,
    ) -> Self {
        Self {
            positional,
            named: named
                .into_iter()
                .map(|(identifier, value)| (identifier.unraw(), value))
                .collect(),
        }
    }

    pub fn get<'s>(
        &'s self,
        format_argument: &FormatArgument<'s>,
    ) -> Result<&'s DeferredFormatValue<'v>, DeferredFormatError> {
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
