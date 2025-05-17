//! # `redefmt-decoder`

// TEMP:
#![allow(missing_docs)]

mod error;
pub(crate) use error::RedefmtDecoderError;

mod core;
pub use core::RedefmtDecoder;

mod decoder_stage;
pub(crate) use decoder_stage::*;

mod frame;
pub(crate) use frame::{DecodedSegment, RedefmtFrame};

mod value;
pub(crate) use value::Value;

mod stores;
pub(crate) use stores::DecoderStores;

mod cache;
pub(crate) use cache::*;

mod segment_decoder;
pub(crate) use segment_decoder::SegmentsDecoder;

mod value_decoder;
pub(crate) use value_decoder::ValueDecoder;

mod list_decoder;
pub(crate) use list_decoder::ListValueDecoder;

mod write_statement_decoder;
pub(crate) use write_statement_decoder::WriteStatementDecoder;

mod utils;
pub(crate) use utils::DecoderUtils;
