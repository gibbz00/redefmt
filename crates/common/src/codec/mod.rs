//! Sometimes referred to as the "wire-format", even it it can be written to any writer.

// frame = <header>[<stamp>](content)*
// content =
// <write_content_id>(<type_hint>[<length_hint>](<type_hint>{1,2})[<type_bytes>])*
// header: u8
// - pointer width
// - stamp included
// type hint u8: primitives (13), collection: (4), write_statement_id (1)
// - of the primitives, only the string needs to send a length hint
// - of the collections, all send a length hint, and a type new hint,
// - the map collection sends an extra type hint
// - write_statement_id will in turn tell if if is a write_statement or type_structure

mod header;
pub use header::Header;

mod stamp;
pub use stamp::Stamp;

mod write_content;
pub use write_content::WriteContentId;

mod type_hint;
pub use type_hint::TypeHint;

mod length_hint;
pub use length_hint::LengthHint;
