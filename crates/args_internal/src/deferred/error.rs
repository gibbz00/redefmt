use crate::ArgumentIdentifier;

#[derive(Debug, thiserror::Error)]
pub enum DeferredFormatError {
    #[error("format positional argument index not present in provided values slice")]
    PositionalOutOfBounds(usize),
    #[error("missing named identifier, or its raw counterpart, in provided arguments")]
    MissingNamed(ArgumentIdentifier<'static>),
}
