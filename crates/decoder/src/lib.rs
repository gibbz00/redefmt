//! # `redefmt-decoder`

// TEMP:
#![allow(missing_docs)]

mod error;
pub(crate) use error::RedefmtDecoderError;

mod core;
pub use core::RedefmtDecoder;

mod decoder_stage;
pub(crate) use decoder_stage::*;

mod write_statement_stage;
pub(crate) use write_statement_stage::{CrateContext, WriteStatementWants};

mod frame;
pub(crate) use frame::{DecodedSegment, RedefmtFrame};

mod value;
pub(crate) use value::Value;

mod stores;
pub(crate) use stores::DecoderStores;

mod crate_cache;
pub(crate) use crate_cache::{CrateCache, CrateValue};

mod statement_cache;
pub(crate) use statement_cache::StatementCache;

mod segment_context;
pub(crate) use segment_context::SegmentContext;

mod value_context;
pub(crate) use value_context::ValueContext;

mod list_context;
pub(crate) use list_context::ListValueContext;

mod utils;
pub(crate) use utils::DecoderUtils;
