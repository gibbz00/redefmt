use crate::*;

#[derive(Debug, thiserror::Error)]
pub enum DeferredFormatError {
    #[error("format positional argument index not present in provided values slice")]
    PositionalOutOfBounds(usize),
    #[error("missing named identifier or its raw counterpart in provided values")]
    MissingNamed(ArgumentIdentifier<'static>),
    #[error("format trait {0:?} not implemented for {1}")]
    FormatNotImplemented(FormatTrait, DeferredValueDiscriminant),
    #[error("invalid argument type received, expected:{0}, received {1}")]
    InvalidArgType(DeferredValueDiscriminant, DeferredValueDiscriminant),
    #[error("failed to convert {0} into an usize")]
    UsizeConversion(DeferredValueDiscriminant),
}
