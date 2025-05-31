//! # `redefmt-decoder`

// TEMP:
#![allow(missing_docs)]

mod error;
pub use error::RedefmtDecoderError;

pub mod value;
pub(crate) use value::{Value, StructVariantValue, TypeStructureValue, TypeStructureVariantValue};

mod stores;
pub(crate) use stores::Stores;

mod cache;
pub use cache::RedefmtDecoderCache;
pub(crate) use cache::*;

mod frame;
pub(crate) use frame::*;
pub use frame::{RedefmtDecoder, RedefmtFrame};

mod sub_decoders;
pub(crate) use sub_decoders::*;

mod utils;
pub(crate) use utils::DecoderUtils;
