use alloc::vec::Vec;

use crate::*;

pub struct Args {
    pub positional: Vec<PositionalArg>,
    // named args after positional, all else is an error
    pub named: Vec<NamedArg>,
}

mod positional;
pub(crate) use positional::PositionalArg;

mod named;
pub(crate) use named::NamedArg;

mod value;
pub(crate) use value::ArgValue;
