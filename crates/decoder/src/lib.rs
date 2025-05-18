//! # `redefmt-decoder`

// TEMP:
#![allow(missing_docs)]

mod error;
pub(crate) use error::RedefmtDecoderError;

mod value;
pub(crate) use value::{ComplexValue, Value};

mod stores;
pub(crate) use stores::Stores;

mod cache;
pub(crate) use cache::*;

mod frame;
pub(crate) use frame::*;

mod segment_decoder;
pub(crate) use segment_decoder::{DecodedSegment, SegmentsDecoder};

mod value_decoder;
pub(crate) use value_decoder::ValueDecoder;

mod list_decoder;
pub(crate) use list_decoder::ListValueDecoder;

mod write_statement_decoder;
pub(crate) use write_statement_decoder::WriteStatementDecoder;

mod type_structure_decoder;
pub(crate) use type_structure_decoder::TypeStructureDecoder;

mod utils;
pub(crate) use utils::DecoderUtils;
