//! Sometimes referred to as the "wire-format", even it it can be written to any writer.

mod header;
pub use header::Header;

mod stamp;
pub use stamp::Stamp;

mod type_hint;
pub use type_hint::TypeHint;

mod length_hint;
pub use length_hint::LengthHint;
