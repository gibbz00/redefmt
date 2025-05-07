//! # `redefmt-decoder`

// TEMP:
#![allow(missing_docs)]

mod core;
pub use core::{RedefmtDecoder, RedefmtDecoderError};

mod frame;
pub(crate) use frame::{DecodedSegment, RedefmtFrame};

mod value;
pub(crate) use value::Value;
