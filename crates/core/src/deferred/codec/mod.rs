//! Sometimes referred to as the "wire-format"

pub mod frame;
pub(crate) use frame::*;

mod dispatcher;
pub use dispatcher::Dispatcher;
#[cfg(test)]
pub use dispatcher::NoopTestDispatcher;
#[cfg(feature = "deferred-testing")]
pub use dispatcher::{SharedTestDispatcher, SimpleTestDispatcher};

mod write_value;
pub use write_value::WriteValue;

mod statement_writer;
pub use statement_writer::{StatementWriter, StatementWriterHint};
