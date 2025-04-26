//! redefmt commons.

#![no_std]
// TEMP:
#![allow(missing_docs)]

extern crate alloc;

mod value;
pub(crate) use value::*;

mod display_segment;
pub(crate) use display_segment::DisplaySegment;

mod documents;
pub(crate) use documents::*;
