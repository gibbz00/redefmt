//! Sometimes referred to as the "wire-format", even it it can be written to any writer.

mod header;
pub use header::Header;

mod pointer_width;
pub use pointer_width::PointerWidth;

mod stamp;
pub use stamp::Stamp;

mod type_hint;
pub use type_hint::TypeHint;
