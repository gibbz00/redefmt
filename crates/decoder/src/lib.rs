//! # `redefmt-decoder`

// TEMP:
#![allow(missing_docs)]

mod error;
pub use error::RedefmtDecoderError;

pub mod values;
pub(crate) use values::{DecodedValues, StructVariantValue, TypeStructureValue, TypeStructureVariantValue, Value};

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
