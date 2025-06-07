//! Sometimes referred to as the "wire-format"

pub mod frame;
pub(crate) use frame::*;

mod write_value;
pub use write_value::WriteValue;

mod statement_hint;
pub use statement_hint::StatementWriterHint;
