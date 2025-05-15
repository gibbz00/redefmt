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
    use redefmt_common::codec::frame::TypeHint;

    use crate::*;

    #[derive(Default)]
    pub struct ValueContext {
        pub length: Option<usize>,
        pub inner_type_hint: Option<TypeHint>,
        pub inner_value_context: InnerContext,
        pub buffer: ValueBuffer,
    }

    #[derive(Default)]
    pub struct ValueBuffer {
        inner: Option<Vec<Value>>,
    }

    impl ValueBuffer {
        pub fn get_or_init(&mut self, capacity: usize) -> &mut Vec<Value> {
            // Unsure of a better way to work around the borrow checker
            let buffer = self.inner.take().unwrap_or_else(|| Vec::with_capacity(capacity));
            self.inner = Some(buffer);
            self.inner.as_mut().expect("buffer not set")
        }

        pub fn clear(&mut self) -> Vec<Value> {
            self.inner.take().unwrap_or_default()
        }
    }

    #[derive(Default)]
    pub struct InnerContext {
        inner: Option<Box<ValueContext>>,
    }

    impl InnerContext {
        pub fn get_or_init(&mut self) -> &mut ValueContext {
            // Unsure of a better way to work around the borrow checker
            let inner_context = self.inner.take().unwrap_or_default();
            self.inner = Some(inner_context);
            self.inner.as_mut().expect("inner value context not set")
        }
    }
}
pub(crate) use value_context::ValueContext;
