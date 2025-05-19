use alloc::vec::Vec;

use crate::*;

pub struct Args {
    pub positional: Vec<PositionalArg>,
    // named args after positional, all else is an error
    pub named: Vec<NamedArg>,
}

pub struct PositionalArg {
    pub value: ArgValue,
}

pub struct NamedArg {
    // important to parse with simply `syn::Ident` here
    // this one is `IDENTIFIER_OR_KEYWORD`
    pub name: Identifier<'static>,
    pub value: ArgValue,
}

mod value;
pub(crate) use value::ArgValue;
