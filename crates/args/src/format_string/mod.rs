mod core;
pub use core::FormatString;
pub(crate) use core::FormatStringSegment;

mod parse;
pub use parse::FormatStringParseError;

mod argument_register;
pub(crate) use argument_register::ArgumentRegister;
pub use argument_register::{RequiredArguments, RequiredArgumentsError};
