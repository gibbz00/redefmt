//! # `redefmt-decoder`

// TEMP:
#![allow(missing_docs)]

mod error;
pub(crate) use error::RedefmtDecoderError;

mod core;
pub use core::RedefmtDecoder;

mod frame;
pub(crate) use frame::{DecodedSegment, RedefmtFrame};

mod value;
pub(crate) use value::Value;

mod crate_cache;
pub(crate) use crate_cache::{CrateCache, CrateValue};

mod print_statement_cache;
pub(crate) use print_statement_cache::PrintStatementCache;

mod value_context {
    #[derive(Default)]
    pub struct ValueContext {
        pub length: Option<u64>,
    }
}
pub(crate) use value_context::ValueContext;
