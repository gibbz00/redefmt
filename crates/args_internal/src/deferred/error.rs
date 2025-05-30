use crate::*;

#[derive(Debug, thiserror::Error)]
pub enum DeferredFormatError {
    #[error("format positional argument index not present in provided values slice")]
    PositionalOutOfBounds(usize),
    #[error("missing named identifier, or its raw counterpart, in provided arguments")]
    MissingNamed(ArgumentIdentifier<'static>),
    #[error("format trait {0:?} not implemented for {1}")]
    FormatNotImplemented(FormatTrait, DeferredValueDiscriminants),
    #[error("invalid argument type received, expected:{0}, received {1}")]
    InvalidArgType(DeferredValueDiscriminants, DeferredValueDiscriminants),
}
