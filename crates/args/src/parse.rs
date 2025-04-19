use alloc::boxed::Box;
use core::{error::Error, ops::Range};

pub trait Parse: Sized {
    fn parse(str: &str) -> Result<Self, ParseError>;
}

#[derive(Debug)]
pub struct ParseError {
    range: Range<usize>,
    kind: Box<dyn Error>,
}

impl ParseError {
    pub(crate) fn new(range: Range<usize>, kind: impl Error + 'static) -> Self {
        Self { range, kind: Box::new(kind) }
    }
}
